---
doc_class: ProductRequirements
template_id: TPL-PRD
prd_id: PRD-erp-coverage
product: erp-coverage
status: Draft
date: 2026-05-20
owner: axis-product + axis-architecture + axis-erp-parity
owner_team: axis-product + axis-architecture + axis-erp-parity
related_oyatie_adrs:
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0251
  - ADR-0255
  - ADR-0263
  - ADR-0313
  - ADR-0314
  - ADR-0315
  - ADR-0316
related_microservices:
  - marketplace
  - payments
  - finops-portal
  - treasury
  - warehouse
  - supply-chain-planning
  - workflow-engine
  - workflow-studio
  - ontology
  - policy-engine
tenant_class: ["evaluation_limited", "paid"]
related_adrs:
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0313
  - ADR-0314
  - ADR-0315
companion_docs:
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
planned_enforcement_ref: D-GOVERNANCE-CENTRAL
live_readiness_claim: target_non_claim_until_presubmit_evidence
---

# PRD: ERP Coverage - SAP-Parity Composition

> **Path convention:** machine-readable service specs use `specs/microservices/*.json`; repo-local implementation and service docs use `oya/<service>/...`. Destination rows use only those two locator forms so schema references and implementation references remain unambiguous.

## A. Problem

Enterprises, growth businesses, and regulated small businesses need ERP-grade financial, operational, supply-chain, workforce, customer, and compliance control without adopting a monolithic suite that traps their tenant identity and business objects. oyatie must cover the SAP S/4HANA module taxonomy while preserving flat microservice ownership, tenant portability, Cedar authorization, workflow-engine orchestration, ontology typing, and marketplace DealSet settlement.
The product is not an ERP platform. The product is a coverage doctrine and set of composable module packs that make oyatie viable for SAP, Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS displacement.

## B. Target Users

### Persona 1 - Marcus, enterprise transformation lead
Marcus runs a multi-subsidiary manufacturer adopting full ERP. He needs FI/CO, MM, SD, PP, QM, PM, EWM, GTS, TRM, and industry packs without collapsing sovereign subsidiary tenants. His frustration is SAP custom code and three-year migrations. Success means every subsidiary keeps identity continuity while parent-level consolidation works.

### Persona 2 - Yejin, side-business founder becoming an enterprise
Yejin starts with marketplace, payments, CRM, inventory, basic finance, and workflow automations. As she grows, she activates warehouse, treasury, quality, supply-chain planning, and HR/payroll packs without migrating to another suite. Success means the same tenant, identities, audit trails, products, customers, and suppliers continue from small business to enterprise.

### Persona 3 - Tomás, LGPD-scale small-business operator
Tomás operates in Brazil with LGPD, tax, e-invoicing, payroll, and supplier obligations. He needs strict data residency, auditable consent, tax compliance, and low operational overhead. Success means ERP-grade controls are activated through packs without exposing him to enterprise-only complexity.

### Persona 4 - Priya, CFO and SOX owner
Priya needs general ledger, period close, AP, AR, treasury, cost centers, project costing, fixed assets, audit evidence, and SOX 404 control proofs. She cares about reconciliation, segregation of duties, and immutable audit-chain evidence.

### Persona 5 - Hana, plant operations director
Hana needs BOM, MRP, capacity, quality holds, plant maintenance, warehouse execution, and supplier performance. She measures uptime, throughput, scrap, and on-time delivery.

### Persona 6 - Alejandra, compliance and trade officer
Alejandra needs GTS export control, sanctions screening, EHS incident evidence, customs, hazardous-material controls, SOC 2, ISO 27001, tax compliance, and jurisdictional pack proof.

## C. User Stories

### Story ERP-001 - FI General Ledger
As a tenant operator responsible for Financial Accounting, I want General Ledger represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the FI General Ledger flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are specs/microservices/accounting.json, oya/payments, oya/finops-portal, oya/treasury and the known gap is documented as "Accounting is spec-present but not a top-level scaffold in the live microservice tree; treasury fills advanced liquidity, debt, and FX gaps.".

### Story ERP-002 - FI Accounts Receivable
As a tenant operator responsible for Financial Accounting, I want Accounts Receivable represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the FI Accounts Receivable flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are specs/microservices/accounting.json, oya/payments, oya/finops-portal, oya/treasury and the known gap is documented as "Accounting is spec-present but not a top-level scaffold in the live microservice tree; treasury fills advanced liquidity, debt, and FX gaps.".

### Story ERP-003 - FI Accounts Payable
As a tenant operator responsible for Financial Accounting, I want Accounts Payable represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the FI Accounts Payable flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are specs/microservices/accounting.json, oya/payments, oya/finops-portal, oya/treasury and the known gap is documented as "Accounting is spec-present but not a top-level scaffold in the live microservice tree; treasury fills advanced liquidity, debt, and FX gaps.".

### Story ERP-004 - FI Fixed Assets
As a tenant operator responsible for Financial Accounting, I want Fixed Assets represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the FI Fixed Assets flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are specs/microservices/accounting.json, oya/payments, oya/finops-portal, oya/treasury and the known gap is documented as "Accounting is spec-present but not a top-level scaffold in the live microservice tree; treasury fills advanced liquidity, debt, and FX gaps.".

### Story ERP-005 - FI Bank Accounting
As a tenant operator responsible for Financial Accounting, I want Bank Accounting represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the FI Bank Accounting flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are specs/microservices/accounting.json, oya/payments, oya/finops-portal, oya/treasury and the known gap is documented as "Accounting is spec-present but not a top-level scaffold in the live microservice tree; treasury fills advanced liquidity, debt, and FX gaps.".

### Story ERP-006 - CO Cost Centers
As a tenant operator responsible for Controlling, I want Cost Centers represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the CO Cost Centers flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/finops-portal, oya/ontology, oya/workflow-engine, oya/supply-chain-planning and the known gap is documented as "Product costing becomes stronger after production-planning and supply-chain-planning publish cost object events.".

### Story ERP-007 - CO Internal Orders
As a tenant operator responsible for Controlling, I want Internal Orders represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the CO Internal Orders flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/finops-portal, oya/ontology, oya/workflow-engine, oya/supply-chain-planning and the known gap is documented as "Product costing becomes stronger after production-planning and supply-chain-planning publish cost object events.".

### Story ERP-008 - CO Profit Centers
As a tenant operator responsible for Controlling, I want Profit Centers represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the CO Profit Centers flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/finops-portal, oya/ontology, oya/workflow-engine, oya/supply-chain-planning and the known gap is documented as "Product costing becomes stronger after production-planning and supply-chain-planning publish cost object events.".

### Story ERP-009 - CO CO-PA
As a tenant operator responsible for Controlling, I want CO-PA represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the CO CO-PA flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/finops-portal, oya/ontology, oya/workflow-engine, oya/supply-chain-planning and the known gap is documented as "Product costing becomes stronger after production-planning and supply-chain-planning publish cost object events.".

### Story ERP-010 - CO Product Costing
As a tenant operator responsible for Controlling, I want Product Costing represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the CO Product Costing flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/finops-portal, oya/ontology, oya/workflow-engine, oya/supply-chain-planning and the known gap is documented as "Product costing becomes stronger after production-planning and supply-chain-planning publish cost object events.".

### Story ERP-011 - MM Procurement
As a tenant operator responsible for Materials Management, I want Procurement represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the MM Procurement flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/workflow-engine, oya/connector, oya/warehouse and the known gap is documented as "Existing marketplace directory is thin; procurement and goods-receipt parity require warehouse and marketplace settlement doctrine.".

### Story ERP-012 - MM Inventory Management
As a tenant operator responsible for Materials Management, I want Inventory Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the MM Inventory Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/workflow-engine, oya/connector, oya/warehouse and the known gap is documented as "Existing marketplace directory is thin; procurement and goods-receipt parity require warehouse and marketplace settlement doctrine.".

### Story ERP-013 - MM Goods Receipt
As a tenant operator responsible for Materials Management, I want Goods Receipt represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the MM Goods Receipt flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/workflow-engine, oya/connector, oya/warehouse and the known gap is documented as "Existing marketplace directory is thin; procurement and goods-receipt parity require warehouse and marketplace settlement doctrine.".

### Story ERP-014 - MM Vendor Master Data
As a tenant operator responsible for Materials Management, I want Vendor Master Data represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the MM Vendor Master Data flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/workflow-engine, oya/connector, oya/warehouse and the known gap is documented as "Existing marketplace directory is thin; procurement and goods-receipt parity require warehouse and marketplace settlement doctrine.".

### Story ERP-015 - MM Purchase Requisitions
As a tenant operator responsible for Materials Management, I want Purchase Requisitions represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the MM Purchase Requisitions flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/workflow-engine, oya/connector, oya/warehouse and the known gap is documented as "Existing marketplace directory is thin; procurement and goods-receipt parity require warehouse and marketplace settlement doctrine.".

### Story ERP-016 - SD Sales Orders
As a tenant operator responsible for Sales and Distribution, I want Sales Orders represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SD Sales Orders flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/payments, oya/crm, oya/warehouse and the known gap is documented as "Customer master, quote-to-cash, credit, and service journeys require crm plus warehouse fulfillment events.".

### Story ERP-017 - SD Pricing
As a tenant operator responsible for Sales and Distribution, I want Pricing represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SD Pricing flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/payments, oya/crm, oya/warehouse and the known gap is documented as "Customer master, quote-to-cash, credit, and service journeys require crm plus warehouse fulfillment events.".

### Story ERP-018 - SD Deliveries
As a tenant operator responsible for Sales and Distribution, I want Deliveries represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SD Deliveries flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/payments, oya/crm, oya/warehouse and the known gap is documented as "Customer master, quote-to-cash, credit, and service journeys require crm plus warehouse fulfillment events.".

### Story ERP-019 - SD Billing
As a tenant operator responsible for Sales and Distribution, I want Billing represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SD Billing flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/payments, oya/crm, oya/warehouse and the known gap is documented as "Customer master, quote-to-cash, credit, and service journeys require crm plus warehouse fulfillment events.".

### Story ERP-020 - SD Credit Management
As a tenant operator responsible for Sales and Distribution, I want Credit Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SD Credit Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/payments, oya/crm, oya/warehouse and the known gap is documented as "Customer master, quote-to-cash, credit, and service journeys require crm plus warehouse fulfillment events.".

### Story ERP-021 - PP BOM
As a tenant operator responsible for Production Planning, I want BOM represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PP BOM flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/production-planning and the known gap is documented as "No existing dedicated production planning microservice.".

### Story ERP-022 - PP MRP
As a tenant operator responsible for Production Planning, I want MRP represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PP MRP flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/production-planning and the known gap is documented as "No existing dedicated production planning microservice.".

### Story ERP-023 - PP Capacity Planning
As a tenant operator responsible for Production Planning, I want Capacity Planning represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PP Capacity Planning flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/production-planning and the known gap is documented as "No existing dedicated production planning microservice.".

### Story ERP-024 - PP Shop Floor
As a tenant operator responsible for Production Planning, I want Shop Floor represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PP Shop Floor flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/production-planning and the known gap is documented as "No existing dedicated production planning microservice.".

### Story ERP-025 - PP Routing
As a tenant operator responsible for Production Planning, I want Routing represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PP Routing flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/production-planning and the known gap is documented as "No existing dedicated production planning microservice.".

### Story ERP-026 - QM Inspection Plans
As a tenant operator responsible for Quality Management, I want Inspection Plans represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the QM Inspection Plans flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/quality-management and the known gap is documented as "No existing dedicated quality management microservice.".

### Story ERP-027 - QM Certificates of Analysis
As a tenant operator responsible for Quality Management, I want Certificates of Analysis represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the QM Certificates of Analysis flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/quality-management and the known gap is documented as "No existing dedicated quality management microservice.".

### Story ERP-028 - QM Quality Notifications
As a tenant operator responsible for Quality Management, I want Quality Notifications represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the QM Quality Notifications flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/quality-management and the known gap is documented as "No existing dedicated quality management microservice.".

### Story ERP-029 - QM Audit Management
As a tenant operator responsible for Quality Management, I want Audit Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the QM Audit Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/quality-management and the known gap is documented as "No existing dedicated quality management microservice.".

### Story ERP-030 - PM Equipment Master
As a tenant operator responsible for Plant Maintenance, I want Equipment Master represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PM Equipment Master flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/plant-maintenance and the known gap is documented as "No existing dedicated plant maintenance microservice.".

### Story ERP-031 - PM Work Orders
As a tenant operator responsible for Plant Maintenance, I want Work Orders represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PM Work Orders flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/plant-maintenance and the known gap is documented as "No existing dedicated plant maintenance microservice.".

### Story ERP-032 - PM Preventive Maintenance
As a tenant operator responsible for Plant Maintenance, I want Preventive Maintenance represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PM Preventive Maintenance flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/plant-maintenance and the known gap is documented as "No existing dedicated plant maintenance microservice.".

### Story ERP-033 - PM Spare Parts
As a tenant operator responsible for Plant Maintenance, I want Spare Parts represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PM Spare Parts flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/plant-maintenance and the known gap is documented as "No existing dedicated plant maintenance microservice.".

### Story ERP-034 - HCM Organizational Management
As a tenant operator responsible for Human Capital Management, I want Organizational Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the HCM Organizational Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are specs/microservices/hr.json, specs/microservices/payroll.json, docs/products/workplace-integration/PRD.md, oya/workflow-engine and the known gap is documented as "HR and payroll are spec-present but not in the 9-service ERP parity scaffold; SuccessFactors-level talent depth remains pack-overlay driven.".

### Story ERP-035 - HCM Personnel Administration
As a tenant operator responsible for Human Capital Management, I want Personnel Administration represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the HCM Personnel Administration flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are specs/microservices/hr.json, specs/microservices/payroll.json, docs/products/workplace-integration/PRD.md, oya/workflow-engine and the known gap is documented as "HR and payroll are spec-present but not in the 9-service ERP parity scaffold; SuccessFactors-level talent depth remains pack-overlay driven.".

### Story ERP-036 - HCM Time Management
As a tenant operator responsible for Human Capital Management, I want Time Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the HCM Time Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are specs/microservices/hr.json, specs/microservices/payroll.json, docs/products/workplace-integration/PRD.md, oya/workflow-engine and the known gap is documented as "HR and payroll are spec-present but not in the 9-service ERP parity scaffold; SuccessFactors-level talent depth remains pack-overlay driven.".

### Story ERP-037 - HCM Payroll
As a tenant operator responsible for Human Capital Management, I want Payroll represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the HCM Payroll flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are specs/microservices/hr.json, specs/microservices/payroll.json, docs/products/workplace-integration/PRD.md, oya/workflow-engine and the known gap is documented as "HR and payroll are spec-present but not in the 9-service ERP parity scaffold; SuccessFactors-level talent depth remains pack-overlay driven.".

### Story ERP-038 - HCM Talent Management
As a tenant operator responsible for Human Capital Management, I want Talent Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the HCM Talent Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are specs/microservices/hr.json, specs/microservices/payroll.json, docs/products/workplace-integration/PRD.md, oya/workflow-engine and the known gap is documented as "HR and payroll are spec-present but not in the 9-service ERP parity scaffold; SuccessFactors-level talent depth remains pack-overlay driven.".

### Story ERP-039 - PS Work Breakdown Structure
As a tenant operator responsible for Project System, I want Work Breakdown Structure represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PS Work Breakdown Structure flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/workflow-engine, oya/ontology, oya/finops-portal, oya/payments and the known gap is documented as "No dedicated PS service is required for first parity; WBS is an ontology object type and workflow template family.".

### Story ERP-040 - PS Networks
As a tenant operator responsible for Project System, I want Networks represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PS Networks flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/workflow-engine, oya/ontology, oya/finops-portal, oya/payments and the known gap is documented as "No dedicated PS service is required for first parity; WBS is an ontology object type and workflow template family.".

### Story ERP-041 - PS Milestone Billing
As a tenant operator responsible for Project System, I want Milestone Billing represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PS Milestone Billing flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/workflow-engine, oya/ontology, oya/finops-portal, oya/payments and the known gap is documented as "No dedicated PS service is required for first parity; WBS is an ontology object type and workflow template family.".

### Story ERP-042 - PS Project Cost Management
As a tenant operator responsible for Project System, I want Project Cost Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PS Project Cost Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/workflow-engine, oya/ontology, oya/finops-portal, oya/payments and the known gap is documented as "No dedicated PS service is required for first parity; WBS is an ontology object type and workflow template family.".

### Story ERP-043 - PLM Master Data Governance
As a tenant operator responsible for Product Lifecycle Management, I want Master Data Governance represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PLM Master Data Governance flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/ontology, oya/workflow-engine, oya/connector, oya/production-planning and the known gap is documented as "Engineering-change specialization is a pack overlay unless production complexity forces a later split.".

### Story ERP-044 - PLM Engineering Change Management
As a tenant operator responsible for Product Lifecycle Management, I want Engineering Change Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PLM Engineering Change Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/ontology, oya/workflow-engine, oya/connector, oya/production-planning and the known gap is documented as "Engineering-change specialization is a pack overlay unless production complexity forces a later split.".

### Story ERP-045 - EHS Hazardous Substances
As a tenant operator responsible for Environment, Health and Safety, I want Hazardous Substances represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the EHS Hazardous Substances flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/compliance, oya/workflow-engine, oya/ontology, oya/quality-management and the known gap is documented as "Regulated incident and hazardous-material packs must add jurisdictional evidence catalogs.".

### Story ERP-046 - EHS Industrial Hygiene
As a tenant operator responsible for Environment, Health and Safety, I want Industrial Hygiene represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the EHS Industrial Hygiene flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/compliance, oya/workflow-engine, oya/ontology, oya/quality-management and the known gap is documented as "Regulated incident and hazardous-material packs must add jurisdictional evidence catalogs.".

### Story ERP-047 - EHS Incident Management
As a tenant operator responsible for Environment, Health and Safety, I want Incident Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the EHS Incident Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/compliance, oya/workflow-engine, oya/ontology, oya/quality-management and the known gap is documented as "Regulated incident and hazardous-material packs must add jurisdictional evidence catalogs.".

### Story ERP-048 - SRM Sourcing
As a tenant operator responsible for Supplier Relationship Management, I want Sourcing represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SRM Sourcing flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/workflow-engine, oya/ontology, oya/payments and the known gap is documented as "Supplier network effects depend on marketplace universal settlement and supplier ontology classes.".

### Story ERP-049 - SRM Contract Management
As a tenant operator responsible for Supplier Relationship Management, I want Contract Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SRM Contract Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/workflow-engine, oya/ontology, oya/payments and the known gap is documented as "Supplier network effects depend on marketplace universal settlement and supplier ontology classes.".

### Story ERP-050 - SRM Supplier Performance
As a tenant operator responsible for Supplier Relationship Management, I want Supplier Performance represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SRM Supplier Performance flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/workflow-engine, oya/ontology, oya/payments and the known gap is documented as "Supplier network effects depend on marketplace universal settlement and supplier ontology classes.".

### Story ERP-051 - CRM Sales Force
As a tenant operator responsible for Customer Relationship Management, I want Sales Force represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the CRM Sales Force flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/crm, oya/community, oya/marketplace, oya/intelligence and the known gap is documented as "No existing dedicated customer relationship lifecycle microservice.".

### Story ERP-052 - CRM Service
As a tenant operator responsible for Customer Relationship Management, I want Service represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the CRM Service flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/crm, oya/community, oya/marketplace, oya/intelligence and the known gap is documented as "No existing dedicated customer relationship lifecycle microservice.".

### Story ERP-053 - CRM Marketing
As a tenant operator responsible for Customer Relationship Management, I want Marketing represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the CRM Marketing flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/crm, oya/community, oya/marketplace, oya/intelligence and the known gap is documented as "No existing dedicated customer relationship lifecycle microservice.".

### Story ERP-054 - CRM Loyalty
As a tenant operator responsible for Customer Relationship Management, I want Loyalty represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the CRM Loyalty flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/crm, oya/community, oya/marketplace, oya/intelligence and the known gap is documented as "No existing dedicated customer relationship lifecycle microservice.".

### Story ERP-055 - SCM/APO Demand Planning
As a tenant operator responsible for Supply Chain Management / Advanced Planning and Optimization, I want Demand Planning represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SCM/APO Demand Planning flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/supply-chain-planning, oya/production-planning, oya/warehouse and the known gap is documented as "No existing dedicated supply-chain planning microservice.".

### Story ERP-056 - SCM/APO Supply Network Planning
As a tenant operator responsible for Supply Chain Management / Advanced Planning and Optimization, I want Supply Network Planning represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SCM/APO Supply Network Planning flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/supply-chain-planning, oya/production-planning, oya/warehouse and the known gap is documented as "No existing dedicated supply-chain planning microservice.".

### Story ERP-057 - SCM/APO Production Planning/Detailed Scheduling
As a tenant operator responsible for Supply Chain Management / Advanced Planning and Optimization, I want Production Planning/Detailed Scheduling represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SCM/APO Production Planning/Detailed Scheduling flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/supply-chain-planning, oya/production-planning, oya/warehouse and the known gap is documented as "No existing dedicated supply-chain planning microservice.".

### Story ERP-058 - SCM/APO Global ATP
As a tenant operator responsible for Supply Chain Management / Advanced Planning and Optimization, I want Global ATP represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SCM/APO Global ATP flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/supply-chain-planning, oya/production-planning, oya/warehouse and the known gap is documented as "No existing dedicated supply-chain planning microservice.".

### Story ERP-059 - SCM/APO Transportation Planning
As a tenant operator responsible for Supply Chain Management / Advanced Planning and Optimization, I want Transportation Planning represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the SCM/APO Transportation Planning flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/supply-chain-planning, oya/production-planning, oya/warehouse and the known gap is documented as "No existing dedicated supply-chain planning microservice.".

### Story ERP-060 - GTS Customs Management
As a tenant operator responsible for Global Trade Services, I want Customs Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the GTS Customs Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/global-trade, oya/compliance, oya/connector and the known gap is documented as "No existing dedicated global trade microservice.".

### Story ERP-061 - GTS Sanctioned Party Screening
As a tenant operator responsible for Global Trade Services, I want Sanctioned Party Screening represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the GTS Sanctioned Party Screening flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/global-trade, oya/compliance, oya/connector and the known gap is documented as "No existing dedicated global trade microservice.".

### Story ERP-062 - GTS Export Control
As a tenant operator responsible for Global Trade Services, I want Export Control represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the GTS Export Control flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/global-trade, oya/compliance, oya/connector and the known gap is documented as "No existing dedicated global trade microservice.".

### Story ERP-063 - GTS Trade Compliance
As a tenant operator responsible for Global Trade Services, I want Trade Compliance represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the GTS Trade Compliance flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/global-trade, oya/compliance, oya/connector and the known gap is documented as "No existing dedicated global trade microservice.".

### Story ERP-064 - TM Freight Order Management
As a tenant operator responsible for Transportation Management, I want Freight Order Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the TM Freight Order Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/supply-chain-planning, oya/warehouse, oya/marketplace, oya/global-trade and the known gap is documented as "Carrier optimization can split later; first parity composes planning, warehouse, marketplace carrier contracts, and trade compliance.".

### Story ERP-065 - TM Carrier Selection
As a tenant operator responsible for Transportation Management, I want Carrier Selection represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the TM Carrier Selection flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/supply-chain-planning, oya/warehouse, oya/marketplace, oya/global-trade and the known gap is documented as "Carrier optimization can split later; first parity composes planning, warehouse, marketplace carrier contracts, and trade compliance.".

### Story ERP-066 - TM Charge Management
As a tenant operator responsible for Transportation Management, I want Charge Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the TM Charge Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/supply-chain-planning, oya/warehouse, oya/marketplace, oya/global-trade and the known gap is documented as "Carrier optimization can split later; first parity composes planning, warehouse, marketplace carrier contracts, and trade compliance.".

### Story ERP-067 - EWM Inbound Processing
As a tenant operator responsible for Extended Warehouse Management, I want Inbound Processing represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the EWM Inbound Processing flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/warehouse and the known gap is documented as "No existing dedicated warehouse microservice.".

### Story ERP-068 - EWM Outbound Processing
As a tenant operator responsible for Extended Warehouse Management, I want Outbound Processing represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the EWM Outbound Processing flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/warehouse and the known gap is documented as "No existing dedicated warehouse microservice.".

### Story ERP-069 - EWM Slotting
As a tenant operator responsible for Extended Warehouse Management, I want Slotting represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the EWM Slotting flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/warehouse and the known gap is documented as "No existing dedicated warehouse microservice.".

### Story ERP-070 - EWM Yard Management
As a tenant operator responsible for Extended Warehouse Management, I want Yard Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the EWM Yard Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/warehouse and the known gap is documented as "No existing dedicated warehouse microservice.".

### Story ERP-071 - EWM Labor Management
As a tenant operator responsible for Extended Warehouse Management, I want Labor Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the EWM Labor Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/warehouse and the known gap is documented as "No existing dedicated warehouse microservice.".

### Story ERP-072 - TRM Cash Management
As a tenant operator responsible for Treasury and Risk Management, I want Cash Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the TRM Cash Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/treasury, oya/payments, oya/finops-portal and the known gap is documented as "Payments handles rails; treasury handles risk, liquidity, debt, and hedging.".

### Story ERP-073 - TRM Liquidity Planning
As a tenant operator responsible for Treasury and Risk Management, I want Liquidity Planning represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the TRM Liquidity Planning flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/treasury, oya/payments, oya/finops-portal and the known gap is documented as "Payments handles rails; treasury handles risk, liquidity, debt, and hedging.".

### Story ERP-074 - TRM FX Management
As a tenant operator responsible for Treasury and Risk Management, I want FX Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the TRM FX Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/treasury, oya/payments, oya/finops-portal and the known gap is documented as "Payments handles rails; treasury handles risk, liquidity, debt, and hedging.".

### Story ERP-075 - TRM Debt Management
As a tenant operator responsible for Treasury and Risk Management, I want Debt Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the TRM Debt Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/treasury, oya/payments, oya/finops-portal and the known gap is documented as "Payments handles rails; treasury handles risk, liquidity, debt, and hedging.".

### Story ERP-076 - RE-FX Lease Management
As a tenant operator responsible for Real Estate Flexible Management, I want Lease Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the RE-FX Lease Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/real-estate, oya/plant-maintenance, oya/finops-portal and the known gap is documented as "No existing dedicated real estate microservice.".

### Story ERP-077 - RE-FX Facility Management
As a tenant operator responsible for Real Estate Flexible Management, I want Facility Management represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the RE-FX Facility Management flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/real-estate, oya/plant-maintenance, oya/finops-portal and the known gap is documented as "No existing dedicated real estate microservice.".

### Story ERP-078 - IS-* IS-Banking
As a tenant operator responsible for Industry Solutions, I want IS-Banking represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the IS-* IS-Banking flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are packs/industry/*, oya/ontology, oya/workflow-engine, oya/compliance and the known gap is documented as "Industry specialization remains pack overlay to preserve ADR-0132 no-grouping boundaries.".

### Story ERP-079 - IS-* IS-Insurance
As a tenant operator responsible for Industry Solutions, I want IS-Insurance represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the IS-* IS-Insurance flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are packs/industry/*, oya/ontology, oya/workflow-engine, oya/compliance and the known gap is documented as "Industry specialization remains pack overlay to preserve ADR-0132 no-grouping boundaries.".

### Story ERP-080 - IS-* IS-Retail
As a tenant operator responsible for Industry Solutions, I want IS-Retail represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the IS-* IS-Retail flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are packs/industry/*, oya/ontology, oya/workflow-engine, oya/compliance and the known gap is documented as "Industry specialization remains pack overlay to preserve ADR-0132 no-grouping boundaries.".

### Story ERP-081 - IS-* IS-Healthcare
As a tenant operator responsible for Industry Solutions, I want IS-Healthcare represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the IS-* IS-Healthcare flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are packs/industry/*, oya/ontology, oya/workflow-engine, oya/compliance and the known gap is documented as "Industry specialization remains pack overlay to preserve ADR-0132 no-grouping boundaries.".

### Story ERP-082 - IS-* IS-Public-Sector
As a tenant operator responsible for Industry Solutions, I want IS-Public-Sector represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the IS-* IS-Public-Sector flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are packs/industry/*, oya/ontology, oya/workflow-engine, oya/compliance and the known gap is documented as "Industry specialization remains pack overlay to preserve ADR-0132 no-grouping boundaries.".

### Story ERP-083 - NETWORK Ariba
As a tenant operator responsible for Network Products, I want Ariba represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the NETWORK Ariba flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/payments, docs/products/workplace-integration/PRD.md, oya/crm and the known gap is documented as "Network effect quality depends on marketplace settlement and connector depth.".

### Story ERP-084 - NETWORK Concur
As a tenant operator responsible for Network Products, I want Concur represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the NETWORK Concur flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/payments, docs/products/workplace-integration/PRD.md, oya/crm and the known gap is documented as "Network effect quality depends on marketplace settlement and connector depth.".

### Story ERP-085 - NETWORK Fieldglass
As a tenant operator responsible for Network Products, I want Fieldglass represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the NETWORK Fieldglass flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/payments, docs/products/workplace-integration/PRD.md, oya/crm and the known gap is documented as "Network effect quality depends on marketplace settlement and connector depth.".

### Story ERP-086 - NETWORK SuccessFactors
As a tenant operator responsible for Network Products, I want SuccessFactors represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the NETWORK SuccessFactors flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/payments, docs/products/workplace-integration/PRD.md, oya/crm and the known gap is documented as "Network effect quality depends on marketplace settlement and connector depth.".

### Story ERP-087 - NETWORK Hybris/Commerce Cloud
As a tenant operator responsible for Network Products, I want Hybris/Commerce Cloud represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the NETWORK Hybris/Commerce Cloud flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/marketplace, oya/payments, docs/products/workplace-integration/PRD.md, oya/crm and the known gap is documented as "Network effect quality depends on marketplace settlement and connector depth.".

### Story ERP-088 - PLATFORM SAP BTP
As a tenant operator responsible for Platform and Extensibility, I want SAP BTP represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PLATFORM SAP BTP flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/plugin-app-store, oya/developer-sdk, oya/workflow-studio, oya/workflow-engine, oya/ontology and the known gap is documented as "No proprietary ABAP clone; extensibility uses SDK, Workflow Engine, Cedar fragments, and typed ontology actions.".

### Story ERP-089 - PLATFORM SAP CAP
As a tenant operator responsible for Platform and Extensibility, I want SAP CAP represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PLATFORM SAP CAP flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/plugin-app-store, oya/developer-sdk, oya/workflow-studio, oya/workflow-engine, oya/ontology and the known gap is documented as "No proprietary ABAP clone; extensibility uses SDK, Workflow Engine, Cedar fragments, and typed ontology actions.".

### Story ERP-090 - PLATFORM SAP Fiori
As a tenant operator responsible for Platform and Extensibility, I want SAP Fiori represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PLATFORM SAP Fiori flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/plugin-app-store, oya/developer-sdk, oya/workflow-studio, oya/workflow-engine, oya/ontology and the known gap is documented as "No proprietary ABAP clone; extensibility uses SDK, Workflow Engine, Cedar fragments, and typed ontology actions.".

### Story ERP-091 - PLATFORM ABAP
As a tenant operator responsible for Platform and Extensibility, I want ABAP represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the PLATFORM ABAP flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/plugin-app-store, oya/developer-sdk, oya/workflow-studio, oya/workflow-engine, oya/ontology and the known gap is documented as "No proprietary ABAP clone; extensibility uses SDK, Workflow Engine, Cedar fragments, and typed ontology actions.".

### Story ERP-092 - DATA SAP Analytics Cloud
As a tenant operator responsible for Data and Analytics, I want SAP Analytics Cloud represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the DATA SAP Analytics Cloud flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/analytics, oya/ontology, oya/intelligence, oya/observability and the known gap is documented as "Transactional ERP storage remains per-service; analytics uses projections rather than one HANA-style monolith.".

### Story ERP-093 - DATA SAP Datasphere
As a tenant operator responsible for Data and Analytics, I want SAP Datasphere represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the DATA SAP Datasphere flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/analytics, oya/ontology, oya/intelligence, oya/observability and the known gap is documented as "Transactional ERP storage remains per-service; analytics uses projections rather than one HANA-style monolith.".

### Story ERP-094 - DATA SAP HANA
As a tenant operator responsible for Data and Analytics, I want SAP HANA represented as an oyatie capability so that I can run SAP-parity operations without leaving tenant scope.
Acceptance: the DATA SAP HANA flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs.
Acceptance: migration from SAP or a comparator ERP preserves source identifiers, row provenance, and continuity of identity.
Acceptance: the destination services are oya/analytics, oya/ontology, oya/intelligence, oya/observability and the known gap is documented as "Transactional ERP storage remains per-service; analytics uses projections rather than one HANA-style monolith.".

## D. Functional Requirements

### D.1 Module coverage matrix

| Module | Requirement | Destination | Status |
|---|---|---|---|
| FI Financial Accounting | Cover General Ledger, Accounts Receivable, Accounts Payable, Fixed Assets, Bank Accounting, Cash Management with tenant-scoped objects, workflows, policy, audit, and migration paths. | specs/microservices/accounting.json, oya/payments, oya/finops-portal, oya/treasury | partial-existing-plus-new-treasury |
| CO Controlling | Cover Cost Centers, Internal Orders, Profit Centers, CO-PA, Product Costing with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/finops-portal, oya/ontology, oya/workflow-engine, oya/supply-chain-planning | covered-by-composition |
| MM Materials Management | Cover Procurement, Inventory Management, Goods Receipt, Vendor Master Data, Purchase Requisitions with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/marketplace, oya/workflow-engine, oya/connector, oya/warehouse | partial-existing-plus-new-warehouse |
| SD Sales and Distribution | Cover Sales Orders, Pricing, Deliveries, Billing, Credit Management, Customer Master Data with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/marketplace, oya/payments, oya/crm, oya/warehouse | partial-existing-plus-new-crm-and-warehouse |
| PP Production Planning | Cover BOM, MRP, Capacity Planning, Shop Floor, Routing with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/production-planning | new-required |
| QM Quality Management | Cover Inspection Plans, Certificates of Analysis, Quality Notifications, Audit Management with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/quality-management | new-required |
| PM Plant Maintenance | Cover Equipment Master, Work Orders, Preventive Maintenance, Spare Parts with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/plant-maintenance | new-required |
| HCM Human Capital Management | Cover Organizational Management, Personnel Administration, Time Management, Payroll, Talent Management with tenant-scoped objects, workflows, policy, audit, and migration paths. | specs/microservices/hr.json, specs/microservices/payroll.json, docs/products/workplace-integration/PRD.md, oya/workflow-engine | planned-existing-spec-coverage |
| PS Project System | Cover Work Breakdown Structure, Networks, Milestone Billing, Project Cost Management with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/workflow-engine, oya/ontology, oya/finops-portal, oya/payments | covered-by-composition |
| PLM Product Lifecycle Management | Cover Master Data Governance, Engineering Change Management with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/ontology, oya/workflow-engine, oya/connector, oya/production-planning | covered-by-composition |
| EHS Environment, Health and Safety | Cover Hazardous Substances, Industrial Hygiene, Incident Management with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/compliance, oya/workflow-engine, oya/ontology, oya/quality-management | covered-by-composition |
| SRM Supplier Relationship Management | Cover Sourcing, Contract Management, Supplier Performance with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/marketplace, oya/workflow-engine, oya/ontology, oya/payments | covered-by-composition |
| CRM Customer Relationship Management | Cover Sales Force, Service, Marketing, Loyalty with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/crm, oya/community, oya/marketplace, oya/intelligence | new-required |
| SCM/APO Supply Chain Management / Advanced Planning and Optimization | Cover Demand Planning, Supply Network Planning, Production Planning/Detailed Scheduling, Global ATP, Transportation Planning with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/supply-chain-planning, oya/production-planning, oya/warehouse | new-required |
| GTS Global Trade Services | Cover Customs Management, Sanctioned Party Screening, Export Control, Trade Compliance with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/global-trade, oya/compliance, oya/connector | new-required |
| TM Transportation Management | Cover Freight Order Management, Carrier Selection, Charge Management with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/supply-chain-planning, oya/warehouse, oya/marketplace, oya/global-trade | covered-by-initial-composition |
| EWM Extended Warehouse Management | Cover Inbound Processing, Outbound Processing, Slotting, Yard Management, Labor Management with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/warehouse | new-required |
| TRM Treasury and Risk Management | Cover Cash Management, Liquidity Planning, FX Management, Debt Management with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/treasury, oya/payments, oya/finops-portal | new-required |
| RE-FX Real Estate Flexible Management | Cover Lease Management, Facility Management with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/real-estate, oya/plant-maintenance, oya/finops-portal | new-required |
| IS-* Industry Solutions | Cover IS-Banking, IS-Insurance, IS-Retail, IS-Healthcare, IS-Public-Sector, IS-Auto, IS-Utilities, IS-Oil, IS-Pharma with tenant-scoped objects, workflows, policy, audit, and migration paths. | packs/industry/*, oya/ontology, oya/workflow-engine, oya/compliance | pack-overlay |
| NETWORK Network Products | Cover Ariba, Concur, Fieldglass, SuccessFactors, Hybris/Commerce Cloud with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/marketplace, oya/payments, docs/products/workplace-integration/PRD.md, oya/crm | covered-by-composition |
| PLATFORM Platform and Extensibility | Cover SAP BTP, SAP CAP, SAP Fiori, ABAP with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/plugin-app-store, oya/developer-sdk, oya/workflow-studio, oya/workflow-engine, oya/ontology | covered-by-composition |
| DATA Data and Analytics | Cover SAP Analytics Cloud, SAP Datasphere, SAP HANA with tenant-scoped objects, workflows, policy, audit, and migration paths. | oya/analytics, oya/ontology, oya/intelligence, oya/observability | covered-by-composition |

### D.FI Functional coverage
- The Financial Accounting pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each FI document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every FI action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every FI event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for FI.

### D.CO Functional coverage
- The Controlling pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each CO document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every CO action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every CO event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for CO.

### D.MM Functional coverage
- The Materials Management pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each MM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every MM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every MM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for MM.

### D.SD Functional coverage
- The Sales and Distribution pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each SD document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every SD action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every SD event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for SD.

### D.PP Functional coverage
- The Production Planning pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each PP document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every PP action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every PP event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for PP.

### D.QM Functional coverage
- The Quality Management pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each QM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every QM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every QM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for QM.

### D.PM Functional coverage
- The Plant Maintenance pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each PM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every PM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every PM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for PM.

### D.HCM Functional coverage
- The Human Capital Management pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each HCM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every HCM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every HCM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for HCM.

### D.PS Functional coverage
- The Project System pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each PS document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every PS action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every PS event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for PS.

### D.PLM Functional coverage
- The Product Lifecycle Management pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each PLM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every PLM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every PLM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for PLM.

### D.EHS Functional coverage
- The Environment, Health and Safety pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each EHS document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every EHS action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every EHS event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for EHS.

### D.SRM Functional coverage
- The Supplier Relationship Management pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each SRM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every SRM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every SRM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for SRM.

### D.CRM Functional coverage
- The Customer Relationship Management pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each CRM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every CRM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every CRM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for CRM.

### D.SCM/APO Functional coverage
- The Supply Chain Management / Advanced Planning and Optimization pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each SCM/APO document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every SCM/APO action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every SCM/APO event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for SCM/APO.

### D.GTS Functional coverage
- The Global Trade Services pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each GTS document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every GTS action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every GTS event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for GTS.

### D.TM Functional coverage
- The Transportation Management pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each TM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every TM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every TM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for TM.

### D.EWM Functional coverage
- The Extended Warehouse Management pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each EWM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every EWM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every EWM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for EWM.

### D.TRM Functional coverage
- The Treasury and Risk Management pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each TRM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every TRM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every TRM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for TRM.

### D.RE-FX Functional coverage
- The Real Estate Flexible Management pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each RE-FX document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every RE-FX action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every RE-FX event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for RE-FX.

### D.IS-* Functional coverage
- The Industry Solutions pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each IS-* document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every IS-* action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every IS-* event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for IS-*.

### D.NETWORK Functional coverage
- The Network Products pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each NETWORK document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every NETWORK action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every NETWORK event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for NETWORK.

### D.PLATFORM Functional coverage
- The Platform and Extensibility pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each PLATFORM document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every PLATFORM action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every PLATFORM event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for PLATFORM.

### D.DATA Functional coverage
- The Data and Analytics pack must expose command, query, event, migration, reconciliation, and reporting surfaces.
- Source-system migration must support SAP first, then Oracle, Workday, NetSuite, Sage Intacct, Infor, and IFS where the module exists.
- Each DATA document must carry tenant id, sub-scope path, source-system id, version, data class, retention class, and audit-chain pointer.
- Every DATA action must have OpenAPI 3.2.0 command representation or explicit non-HTTP rationale.
- Every DATA event must have AsyncAPI 3.1.0 channel representation or explicit local-only rationale.
- Internal cross-service calls must define proto3 contracts when synchronous semantics are required.
- Pack overlays must declare which Cedar fragments, cell tiers, and sovereign rules activate for DATA.

## E. Non-Functional Requirements

| Dimension | Requirement | Target |
|---|---|---|
| Maintainability | Module ownership is service-local with generated coverage registry. | No ERP platform service; every module maps to owners. |
| Observability | Each ERP document emits metrics, traces, logs, and audit events. | 100 percent of critical document state transitions sealed. |
| Scalability | Module services scale by their natural bottlenecks. | 10x growth without ownership split; 100x through shard/partition expansion. |
| Performance | Interactive commands return fast or become explicit async jobs. | p95 under 300 ms for lightweight commands; p95 under 2 s for queries; batch import async. |
| Optimization | Costs are attributed per tenant, module, document type, and workflow. | Finops export includes module tags. |
| Code quality | Tests cover contracts, migrations, authorization, idempotency, and replay. | >=85 percent line and >=75 percent branch where code exists. |
| Data integrity | Financial, inventory, trade, and payroll records are reversible through domain events. | No destructive correction path. |
| Availability | ERP critical operations degrade explicitly. | 99.9 percent preview, 99.95 percent GA for critical paths. |
| Migration | Every import emits evidence and reconciliation. | Dry-run mismatch rate under tenant-approved threshold. |

Capacity math: if a tenant imports 100 million SAP rows over a 10-hour window, average sustained import is 2,778 rows/second. With validation service time of 20 ms per 100-row batch, Little's Law gives 0.56 active batch workers at average load; production admission uses at least 20 workers per import lane for burst, retries, and checksum validation.

## F. UX Flows

### Flow 1: SAP migration dry run
```text
[1] Admin chooses source SAP system ->
[2] adapter inventories tables ->
[3] Ontology mapping preview renders ->
[4] Dry-run validates counts ->
[5] Evidence report signs ->
[6] Operator schedules cutover
```
Evidence: Flow 1 emits workflow trace, audit-chain events, tenant-scoped metrics, and migration or operation proof as applicable.

### Flow 2: Marcus subsidiary rollout
```text
[1] Parent tenant opens ERP coverage ->
[2] Selects child tenant ->
[3] Activates module pack ->
[4] Cedar grant verifies parent scope ->
[5] Workflow provisions services ->
[6] Child audit stream receives evidence
```
Evidence: Flow 2 emits workflow trace, audit-chain events, tenant-scoped metrics, and migration or operation proof as applicable.

### Flow 3: Yejin growth activation
```text
[1] Starts with marketplace order ->
[2] Adds CRM account ->
[3] Adds warehouse stock ->
[4] Adds finance posting ->
[5] Activates treasury cash view ->
[6] No identity migration occurs
```
Evidence: Flow 3 emits workflow trace, audit-chain events, tenant-scoped metrics, and migration or operation proof as applicable.

### Flow 4: Tomás LGPD pack
```text
[1] Select Brazil pack ->
[2] Residency rules load ->
[3] Tax invoice workflow loads ->
[4] Consent and DSAR rules bind ->
[5] ERP module dashboards filter by tenant ->
[6] Audit proof exports
```
Evidence: Flow 4 emits workflow trace, audit-chain events, tenant-scoped metrics, and migration or operation proof as applicable.

### Flow 5: Quality hold to shipment release
```text
[1] Inspection lot fails ->
[2] Quality hold event emits ->
[3] Warehouse blocks pick ->
[4] Workflow requests approval ->
[5] Corrective action closes ->
[6] Outbound shipment releases
```
Evidence: Flow 5 emits workflow trace, audit-chain events, tenant-scoped metrics, and migration or operation proof as applicable.

### Flow 6: Treasury liquidity close
```text
[1] Bank feed imports ->
[2] Cash position reconciles ->
[3] FX exposure calculates ->
[4] Hedge designation reviewed ->
[5] CFO approves ->
[6] Audit-chain seals close evidence
```
Evidence: Flow 6 emits workflow trace, audit-chain events, tenant-scoped metrics, and migration or operation proof as applicable.

## G. Success Metrics

- Time to value: FI/CO dry-run import within 14 days for a mid-market tenant; full manufacturing PP/QM/PM/EWM pilot within 60 days after data inventory.
- Module activation: Yejin-style growth tenant activates a new module pack in under 30 minutes when no external migration is required.
- Data integrity: zero silent cross-tenant writes; 100 percent of critical document reversals carry audit-chain evidence.
- Transaction performance: p95 lightweight command under 300 ms; p99 document query under 2 s for 95 percent of tenant dashboards; batch imports expose progress within 5 s.
- Migration quality: imported document counts reconcile to source counts, and rejected rows include machine-readable reason, source pointer, and remediation workflow.
- Compliance: SOX 404 control evidence exported on demand; SOC 2 and ISO 27001 controls trace to module events; tax compliance packs declare jurisdictional status.

## H. Compliance Impact

- SOX 404: segregation of duties, approval trails, journal entry evidence, control testing, and change evidence are first-class requirements.
- SOC 2: security, availability, confidentiality, processing integrity, and privacy controls map to audit-chain events and runbooks.
- ISO 27001: asset ownership, access control, cryptography, supplier relationships, incident management, and continuity controls map to module services.
- Tax compliance: per-jurisdiction pack overlays cover VAT/GST/sales tax, withholding, payroll tax, e-invoicing, invoice retention, and audit export.
- LGPD/GDPR/CPRA: data subject access, consent, retention, deletion, and portability remain tenant scoped and source-system traceable.
- Trade compliance: GTS/GTS-like packs include sanctions screening, export control, customs, broker filing, and evidence retention.

## I. Open Questions

- Whether transportation management should split into a dedicated service after supply-chain-planning and warehouse expose first parity evidence.
- Whether HR/payroll spec-present coverage should receive its own Wave-3-G full artifact buildout before or after treasury and global-trade.
- Which regional tax packs should be promoted first after KR, US, EU, and Brazil anchors.
- Which SAP industry solutions receive first pack overlays: IS-Retail, IS-Healthcare, IS-Banking, or IS-Public-Sector.

## J. Out-of-Scope

- A monolithic ERP microservice or ERP platform brand at the architecture layer.
- A proprietary ABAP clone or tenant scripting runtime that bypasses Workflow Engine and Cedar.
- Big-bang migration without dry-run, reconciliation, and rollback evidence.
- Silent import of source-system rows that cannot be mapped to tenant-scoped ontology types.

## K. References

- SAP S/4HANA: https://help.sap.com/docs/SAP_S4HANA_ON-PREMISE
- SAP Ariba: https://help.sap.com/docs/ariba
- SAP Concur: https://www.concur.com/
- SAP Fieldglass: https://www.sap.com/products/fieldglass.html
- SAP SuccessFactors: https://www.sap.com/products/hcm.html
- SAP Commerce Cloud: https://help.sap.com/docs/SAP_COMMERCE_CLOUD_PUBLIC_CLOUD
- SAP BTP: https://help.sap.com/docs/btp
- SAP CAP: https://cap.cloud.sap/docs/
- SAP Fiori: https://experience.sap.com/fiori-design-web/
- SAP HANA: https://help.sap.com/docs/SAP_HANA_PLATFORM
- Oracle Fusion Cloud ERP: https://docs.oracle.com/en/cloud/saas/erp/
- Microsoft Dynamics 365: https://learn.microsoft.com/en-us/dynamics365/
- Workday Financial Management: https://www.workday.com/en-us/products/financial-management.html
- NetSuite ERP: https://www.netsuite.com/portal/products/erp.shtml
- Sage Intacct: https://www.sage.com/en-us/products/sage-intacct/
- Infor CloudSuite: https://www.infor.com/solutions/erp
- IFS Cloud: https://www.ifs.com/cloud
- Stripe Connect: https://docs.stripe.com/connect
- Coupa: https://www.coupa.com/products/
- Salesforce Commerce Cloud: https://www.salesforce.com/products/commerce/
- Gartner ERP MQ reference: https://www.gartner.com/en/documents/
- Forrester Wave ERP reference: https://www.forrester.com/report/
- PRD traceability row 1: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 2: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 3: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 4: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 5: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 6: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 7: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 8: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 9: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 10: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 11: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 12: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 13: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 14: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 15: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 16: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 17: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 18: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 19: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 20: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 21: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 22: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 23: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 24: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 25: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 26: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 27: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 28: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 29: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 30: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 31: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 32: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 33: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 34: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 35: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 36: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 37: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 38: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 39: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 40: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 41: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 42: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 43: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 44: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 45: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 46: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 47: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 48: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 49: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 50: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 51: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 52: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 53: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 54: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 55: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 56: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 57: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 58: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 59: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 60: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 61: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 62: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 63: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 64: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 65: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 66: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 67: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 68: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 69: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 70: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 71: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 72: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 73: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 74: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 75: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 76: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 77: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 78: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 79: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 80: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 81: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 82: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 83: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 84: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 85: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 86: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 87: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 88: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 89: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 90: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 91: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 92: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 93: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 94: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 95: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 96: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 97: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 98: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 99: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 100: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 101: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 102: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 103: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 104: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 105: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 106: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 107: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 108: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 109: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 110: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 111: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 112: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 113: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 114: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 115: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 116: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 117: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 118: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 119: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 120: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 121: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 122: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 123: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 124: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 125: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 126: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 127: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 128: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 129: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 130: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 131: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 132: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 133: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 134: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 135: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 136: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 137: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 138: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 139: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 140: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 141: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 142: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 143: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 144: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 145: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 146: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 147: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 148: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 149: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 150: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 151: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 152: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 153: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 154: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 155: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 156: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 157: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 158: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 159: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 160: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 161: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 162: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 163: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 164: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 165: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 166: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 167: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 168: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 169: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 170: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 171: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 172: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 173: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 174: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 175: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 176: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 177: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 178: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 179: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 180: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 181: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 182: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 183: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 184: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 185: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 186: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 187: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 188: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 189: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 190: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 191: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 192: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 193: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 194: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 195: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 196: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 197: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 198: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 199: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 200: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 201: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 202: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 203: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 204: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 205: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 206: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 207: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 208: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 209: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 210: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 211: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 212: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 213: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 214: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 215: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 216: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 217: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 218: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 219: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 220: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 221: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 222: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 223: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 224: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 225: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 226: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 227: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 228: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 229: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 230: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 231: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 232: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 233: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 234: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 235: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 236: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 237: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 238: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 239: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 240: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 241: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 242: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 243: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 244: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 245: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 246: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 247: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 248: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 249: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 250: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 251: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 252: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 253: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 254: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 255: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 256: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 257: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 258: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 259: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 260: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 261: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 262: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 263: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 264: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 265: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 266: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 267: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 268: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 269: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 270: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 271: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 272: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 273: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 274: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 275: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 276: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 277: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 278: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 279: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 280: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 281: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 282: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 283: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 284: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 285: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 286: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 287: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 288: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 289: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 290: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 291: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 292: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 293: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 294: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 295: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 296: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 297: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 298: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 299: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 300: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 301: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 302: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 303: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 304: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 305: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 306: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 307: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 308: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 309: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 310: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 311: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 312: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 313: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 314: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 315: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 316: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 317: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 318: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 319: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 320: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 321: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 322: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 323: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 324: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 325: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 326: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 327: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 328: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 329: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 330: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 331: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 332: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 333: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 334: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 335: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 336: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 337: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 338: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 339: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 340: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 341: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 342: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 343: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 344: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 345: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 346: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 347: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 348: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 349: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 350: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 351: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 352: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 353: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 354: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 355: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 356: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 357: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 358: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 359: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 360: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 361: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 362: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 363: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 364: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 365: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 366: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 367: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 368: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 369: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 370: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 371: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 372: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 373: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 374: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 375: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 376: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 377: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 378: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 379: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 380: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 381: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 382: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 383: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 384: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 385: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 386: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 387: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 388: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 389: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 390: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 391: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 392: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 393: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 394: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 395: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 396: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 397: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 398: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 399: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 400: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 401: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 402: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 403: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 404: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 405: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 406: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 407: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 408: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 409: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 410: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 411: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 412: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 413: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 414: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 415: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 416: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 417: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 418: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 419: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 420: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 421: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 422: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 423: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 424: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 425: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 426: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 427: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 428: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 429: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 430: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 431: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 432: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 433: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 434: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 435: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 436: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 437: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 438: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 439: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 440: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 441: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 442: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 443: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 444: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 445: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 446: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 447: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 448: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 449: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 450: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 451: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 452: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 453: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 454: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 455: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 456: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 457: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 458: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 459: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 460: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 461: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 462: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 463: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 464: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 465: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 466: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 467: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 468: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 469: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 470: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 471: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 472: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 473: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 474: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 475: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 476: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 477: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 478: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 479: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 480: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 481: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 482: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 483: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 484: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 485: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 486: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 487: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 488: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 489: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 490: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 491: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 492: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 493: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 494: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 495: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 496: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 497: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 498: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 499: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 500: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 501: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 502: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 503: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 504: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 505: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 506: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 507: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 508: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 509: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 510: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 511: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 512: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 513: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 514: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 515: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 516: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 517: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 518: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 519: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 520: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 521: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 522: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 523: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 524: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 525: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 526: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 527: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 528: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 529: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 530: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 531: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 532: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 533: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 534: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 535: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 536: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 537: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 538: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 539: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 540: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 541: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 542: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 543: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 544: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 545: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 546: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 547: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 548: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 549: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 550: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 551: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 552: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 553: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 554: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 555: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 556: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 557: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 558: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 559: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 560: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 561: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 562: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 563: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 564: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 565: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 566: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 567: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 568: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 569: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 570: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 571: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 572: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 573: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 574: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 575: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 576: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 577: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 578: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 579: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 580: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 581: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 582: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 583: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 584: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 585: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 586: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 587: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 588: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 589: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 590: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 591: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 592: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 593: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 594: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 595: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 596: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 597: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 598: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 599: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 600: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 601: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 602: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 603: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 604: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 605: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 606: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 607: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 608: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 609: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 610: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 611: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 612: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 613: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 614: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 615: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 616: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 617: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 618: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 619: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 620: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 621: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 622: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 623: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 624: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 625: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 626: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 627: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 628: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 629: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 630: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 631: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 632: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 633: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 634: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 635: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 636: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 637: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 638: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 639: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 640: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 641: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 642: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 643: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 644: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 645: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 646: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 647: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 648: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 649: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 650: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 651: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 652: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 653: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 654: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 655: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 656: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 657: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 658: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 659: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 660: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 661: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 662: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 663: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 664: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 665: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 666: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 667: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 668: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 669: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 670: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 671: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 672: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 673: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 674: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 675: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 676: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 677: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 678: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 679: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 680: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 681: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 682: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 683: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 684: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 685: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 686: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 687: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 688: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 689: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 690: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 691: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 692: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 693: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 694: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 695: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 696: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 697: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 698: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 699: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 700: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 701: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 702: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 703: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 704: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 705: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 706: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 707: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 708: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 709: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 710: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 711: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 712: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 713: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 714: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 715: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 716: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 717: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 718: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 719: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 720: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 721: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 722: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 723: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 724: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 725: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 726: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 727: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 728: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 729: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 730: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 731: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 732: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 733: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 734: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 735: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 736: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 737: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 738: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 739: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 740: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 741: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 742: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 743: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 744: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 745: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 746: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 747: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 748: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 749: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 750: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 751: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 752: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 753: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 754: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 755: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 756: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 757: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 758: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 759: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 760: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 761: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 762: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 763: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 764: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 765: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 766: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 767: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 768: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 769: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 770: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 771: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 772: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 773: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 774: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 775: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 776: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 777: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 778: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 779: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 780: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 781: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 782: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 783: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 784: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 785: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 786: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 787: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 788: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 789: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 790: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 791: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 792: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 793: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 794: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 795: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 796: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 797: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 798: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 799: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 800: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 801: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 802: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 803: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 804: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 805: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 806: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 807: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 808: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 809: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 810: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 811: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 812: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 813: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 814: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 815: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 816: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 817: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 818: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 819: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 820: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 821: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 822: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 823: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 824: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 825: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 826: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 827: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 828: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 829: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 830: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 831: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 832: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 833: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 834: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 835: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 836: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 837: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 838: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 839: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 840: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 841: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 842: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 843: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 844: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 845: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 846: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 847: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 848: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 849: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 850: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 851: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 852: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 853: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 854: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 855: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 856: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 857: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 858: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 859: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 860: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 861: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 862: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 863: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 864: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 865: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 866: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 867: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 868: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 869: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 870: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 871: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 872: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 873: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 874: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 875: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 876: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 877: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 878: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 879: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 880: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 881: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 882: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 883: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 884: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 885: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 886: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 887: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 888: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 889: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 890: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 891: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 892: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 893: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 894: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 895: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 896: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 897: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 898: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 899: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 900: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 901: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 902: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 903: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 904: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 905: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 906: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 907: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 908: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 909: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 910: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 911: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 912: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 913: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 914: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 915: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 916: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 917: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 918: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 919: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 920: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 921: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 922: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 923: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 924: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 925: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 926: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 927: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 928: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 929: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 930: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 931: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 932: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 933: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 934: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 935: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 936: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 937: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 938: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 939: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 940: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 941: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 942: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 943: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 944: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 945: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 946: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 947: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 948: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 949: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 950: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 951: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 952: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 953: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 954: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 955: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 956: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 957: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 958: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 959: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 960: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 961: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 962: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 963: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 964: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 965: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 966: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 967: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 968: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 969: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 970: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 971: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 972: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 973: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 974: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 975: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 976: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 977: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 978: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 979: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 980: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 981: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 982: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 983: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 984: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 985: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 986: IS-* Industry Solutions remains covered only when packs/industry/* + oya/ontology + oya/workflow-engine + oya/compliance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 987: NETWORK Network Products remains covered only when oya/marketplace + oya/payments + docs/products/workplace-integration/PRD.md + oya/crm publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 988: PLATFORM Platform and Extensibility remains covered only when oya/plugin-app-store + oya/developer-sdk + oya/workflow-studio + oya/workflow-engine + oya/ontology publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 989: DATA Data and Analytics remains covered only when oya/analytics + oya/ontology + oya/intelligence + oya/observability publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 990: FI Financial Accounting remains covered only when specs/microservices/accounting.json + oya/payments + oya/finops-portal + oya/treasury publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 991: CO Controlling remains covered only when oya/finops-portal + oya/ontology + oya/workflow-engine + oya/supply-chain-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 992: MM Materials Management remains covered only when oya/marketplace + oya/workflow-engine + oya/connector + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 993: SD Sales and Distribution remains covered only when oya/marketplace + oya/payments + oya/crm + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 994: PP Production Planning remains covered only when oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 995: QM Quality Management remains covered only when oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 996: PM Plant Maintenance remains covered only when oya/plant-maintenance publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 997: HCM Human Capital Management remains covered only when specs/microservices/hr.json + specs/microservices/payroll.json + docs/products/workplace-integration/PRD.md + oya/workflow-engine publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 998: PS Project System remains covered only when oya/workflow-engine + oya/ontology + oya/finops-portal + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 999: PLM Product Lifecycle Management remains covered only when oya/ontology + oya/workflow-engine + oya/connector + oya/production-planning publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 1000: EHS Environment, Health and Safety remains covered only when oya/compliance + oya/workflow-engine + oya/ontology + oya/quality-management publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 1001: SRM Supplier Relationship Management remains covered only when oya/marketplace + oya/workflow-engine + oya/ontology + oya/payments publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 1002: CRM Customer Relationship Management remains covered only when oya/crm + oya/community + oya/marketplace + oya/intelligence publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 1003: SCM/APO Supply Chain Management / Advanced Planning and Optimization remains covered only when oya/supply-chain-planning + oya/production-planning + oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 1004: GTS Global Trade Services remains covered only when oya/global-trade + oya/compliance + oya/connector publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 1005: TM Transportation Management remains covered only when oya/supply-chain-planning + oya/warehouse + oya/marketplace + oya/global-trade publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 1006: EWM Extended Warehouse Management remains covered only when oya/warehouse publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 1007: TRM Treasury and Risk Management remains covered only when oya/treasury + oya/payments + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.
- PRD traceability row 1008: RE-FX Real Estate Flexible Management remains covered only when oya/real-estate + oya/plant-maintenance + oya/finops-portal publish user-story evidence, policy fragments, migration proof, observability hooks, and pack-overlay activation metadata.

---

## Hero Surface Substance Bar Addendum - ERP Coverage

This addendum is the 2026-05-20 product-doc deepening wave for the ERP hero surface. It keeps the existing SAP-parity catalog above, but adds the missing buildable product layer: named personas, jobs-to-be-done, user stories with pass criteria, surface sketches, data model, Cedar policy model, workflow integration, ADR-0255 two-layer intelligence integration, ADR-0251 pack overlays, ADR-0263 telemetry, migration playbooks, ADR-0316 capability tiers, and cross-product dependencies.

## Vision

ERP Coverage exists so an organization can start with one oyatie tenant and never outgrow the platform when it becomes a multi-subsidiary, multi-jurisdiction, audited enterprise. The product is for CFOs, operations leaders, supply-chain teams, HR leaders, and founders who need SAP-class control without monolithic platform lock-in. The "why now" is that oyatie already owns the universal primitives - tenant, identity, Cedar, workflow, audit chain, ontology, marketplace settlement, and regional packs - so ERP becomes a composed operating model rather than a second system of record.

## Personas

- Primary: Marcus Chen, enterprise transformation lead; see docs/personas/MASTER-ROSTER-2026-05-21.md row 2.
- Primary: CFO Helena Brandt, public-company CFO; see docs/personas/MASTER-ROSTER-2026-05-21.md row 26.
- Primary: Priya Krishnan, HR director; see docs/personas/MASTER-ROSTER-2026-05-21.md row 8.
- Primary: Hana, plant operations director; aligns to production workspace personas in MASTER-ROSTER.
- Secondary: Tomas Garcia, side-business operator growing into enterprise; see row 4.
- Secondary: Carlos Martinez, warehouse field worker; see row 11.
- Secondary: Alejandra, compliance and trade officer; maps to compliance and external-regulator audience types.
- Secondary: Sam Okafor, corporate internal-audit director; see row 9.
- Secondary: Board director Patrick O'Reilly; see row 34.
- Secondary: Yejin Park, side-business owner and healthcare worker; see row 1.

## Jobs-to-be-Done

### Job-to-be-done-ERP-01 - Close the books without leaving the tenant
- Situation: Helena closes a public-company month with subsidiaries in KR, US, EU, and BR.
- Motivation: She needs GL, AP, AR, fixed assets, treasury, and audit evidence in one tenant graph.
- Acceptance: close status is visible per subsidiary, every journal source links to ontology objects, and EVT-ERP-CLOSE-STEP-SEALED emits for every control step.
- Acceptance: Cedar denies a cross-subsidiary journal unless parent-scope approval exists.

### Job-to-be-done-ERP-02 - Move from SAP to oyatie with reversible proof
- Situation: Marcus runs an SAP dry-run migration.
- Motivation: He must prove that source identifiers, authorization, tax treatment, and audit evidence survive migration.
- Acceptance: migration dry run produces object-count parity, rejected-row reasons, rollback checkpoints, and source-to-oyatie lineage.
- Acceptance: pass criteria include zero unclassified master-data rows and zero orphan accounting documents.

### Job-to-be-done-ERP-03 - Procure to pay with supplier governance
- Situation: Hana needs a critical part, the supplier is new, and spend exceeds the local approval threshold.
- Motivation: She needs requisition, vendor onboarding, purchase order, goods receipt, invoice, and payment in one flow.
- Acceptance: workflow-engine runs ReqToPaySaga with KYC, budget, sanctions, goods receipt, invoice match, and payment release nodes.
- Acceptance: Cedar enforces requester, approver, receiving clerk, and payment releaser separation of duties.

### Job-to-be-done-ERP-04 - Make plant quality holds operationally visible
- Situation: A batch fails inspection and must block shipment.
- Motivation: Operations needs fast hold propagation without manual spreadsheet coordination.
- Acceptance: QualityHold object links batch, lot, shipment, customer order, and financial reserve.
- Acceptance: the hold emits EVT-ERP-QUALITY-HOLD-PLACED and blocks delivery release until a signed disposition exists.

### Job-to-be-done-ERP-05 - Convert side-business simplicity into enterprise depth
- Situation: Tomas starts with invoices and inventory, then adds treasury, warehouse, compliance, and HR.
- Motivation: He should activate modules without tenant migration.

### Job-to-be-done-ERP-06 - Govern trade, customs, and sanctions
- Situation: Alejandra ships dual-use goods from KR to EU.
- Motivation: She needs GTS checks at order, shipment, and invoice release.
- Acceptance: export-control classification, sanctioned-party screen, customs document, and evidence export are all linked to the shipment graph.
- Acceptance: Cedar forbids shipment release when sanctions status is stale or unresolved.

### Job-to-be-done-ERP-07 - Forecast cash and liquidity
- Situation: Treasury sees cash concentration risk.
- Motivation: It needs working-capital view across AP, AR, payroll, taxes, debt, and FX.
- Acceptance: treasury projection consumes ERP cash events and emits daily liquidity forecast with confidence band.
- Acceptance: forecast drift > configured threshold opens a workflow-engine investigation.

### Job-to-be-done-ERP-08 - Preserve auditability across every module
- Situation: Sam audits a SOX control across procurement and GL.
- Motivation: He needs immutable evidence without asking module owners for screenshots.
- Acceptance: every module step links to audit-chain event id, policy decision id, workflow run id, and ontology object id.
- Acceptance: audit export redacts PII by pack, but preserves control evidence completeness.

## User Stories

### Story ERP-HS-001 - Consolidated Close Console
As CFO Helena Brandt, I want a close console per entity and currency so that I can complete monthly close with no spreadsheet tracker.
Pass: all subsidiaries show close phase, blocker, owner, aging, and evidence seal.
Pass: EVT-ERP-CLOSE-STEP-SEALED fires for every completed checklist item.

### Story ERP-HS-002 - Journal Lineage
As an internal auditor, I want journal entries to show source document, workflow run, policy decision, and ontology object so that every posting can be reconstructed.
Pass: every journal line has source_ref, workflow_run_id, policy_decision_id, and audit_event_id.
Pass: missing lineage blocks posting.

### Story ERP-HS-003 - Three-Way Match
As AP lead, I want invoice, purchase order, and goods receipt matching so that payment only releases after evidence is complete.
Pass: tolerance thresholds are visible by supplier, entity, and pack.
Pass: mismatch opens ReqToPayExceptionSaga.

### Story ERP-HS-004 - Supplier Onboarding
As procurement owner, I want vendor onboarding to run KYC, tax, sanctions, bank verification, and contract capture so that supplier risk is governed before spend.
Pass: supplier status cannot become active until all mandatory checks are green.
Pass: every failed check includes remediation owner and retry policy.

### Story ERP-HS-005 - Budget Encumbrance
As department manager, I want purchase requests to reserve budget at approval time so that spend is not double-committed.
Pass: budget_hold_id is created before PO issue.
Pass: cancellation releases the hold with audit evidence.

### Story ERP-HS-006 - Inventory Reservation
As warehouse planner, I want sales orders to reserve inventory across cells so that promise dates are honest.
Pass: reservation shows lot, cell, expiry, and substitution option.
Pass: cross-cell reservation requires residency-compatible policy.

### Story ERP-HS-007 - Quality Hold
As plant quality manager, I want failed inspection to block shipment and billing so that defective goods do not leave the plant.
Pass: QualityHold status propagates to shipment and invoice release.
Pass: release requires signed disposition and role separation.

### Story ERP-HS-008 - Production Order Costing
As cost accountant, I want production-order actuals against standard cost so that margin variance is visible.
Pass: material, labor, machine, scrap, and overhead variances are separate lines.
Pass: each variance links to production and inventory objects.

### Story ERP-HS-009 - Treasury Cash Forecast
As treasury analyst, I want AP, AR, payroll, taxes, debt, and FX cash flows in one forecast so that liquidity risk is detected early.
Pass: forecast exposes confidence band and assumptions.
Pass: forecast drift emits EVT-ERP-CASH-FORECAST-DRIFT.

### Story ERP-HS-010 - FX Exposure
As CFO, I want open AR/AP and intercompany exposure grouped by currency so that hedging is evidence-backed.
Pass: exposure is grouped by currency, due bucket, counterparty, and entity.
Pass: hedge recommendation cites source documents.

### Story ERP-HS-011 - Intercompany Elimination
As group controller, I want intercompany balances matched and eliminated so that consolidation is accurate.
Pass: mismatch requires counterparty confirmation.
Pass: elimination journal links both sides and audit evidence.

### Story ERP-HS-012 - Fixed Asset Lifecycle
As asset accountant, I want acquisition, depreciation, impairment, transfer, and disposal in one lifecycle so that asset accounting is complete.
Pass: asset book supports local GAAP and group GAAP.
Pass: disposal blocks if linked lease or insurance record is active.

### Story ERP-HS-013 - Project Cost Control
As project director, I want WBS, budget, commitment, actual, and forecast in one view so that overruns surface early.
Pass: WBS node shows actual, committed, remaining, and estimate-at-completion.
Pass: overrun threshold opens ProjectCostExceptionSaga.

### Story ERP-HS-014 - Trade Compliance
As trade officer, I want export-control and sanctions checks before shipment so that illegal shipment cannot release.
Pass: shipment release checks classification, destination, end user, and license.
Pass: expired screen blocks release.

### Story ERP-HS-015 - Warehouse Wave Picking
As warehouse supervisor, I want wave picking tied to sales orders, carrier cutoff, and lot rules so that outbound work is efficient.
Pass: wave shows route, priority, lot constraints, and handheld tasks.
Pass: mispick emits inventory correction event and opens exception.

### Story ERP-HS-016 - Mobile Receiving
As Carlos Martinez, I want rugged-device receiving so that goods receipt is captured at dock without desktop access.
Pass: device supports scan, photo evidence, discrepancy note, and offline queue.
Pass: sync conflict requires supervisor approval.

### Story ERP-HS-017 - Manufacturing Routing
As production planner, I want routing steps with capacity and machine constraints so that feasible schedules are generated.
Pass: routing step includes work center, setup time, run time, and quality gate.
Pass: overloaded work center triggers reschedule recommendation.

### Story ERP-HS-018 - HR Cost Center Sync
As HR director Priya, I want org changes to update cost center assignments so that payroll and accounting agree.
Pass: employee transfer creates effective-dated cost allocation.
Pass: payroll run blocks on unresolved cost center.

### Story ERP-HS-019 - Audit Evidence Export
As Sam Okafor, I want control evidence export by period and process so that SOX testing does not require screenshots.
Pass: export includes event ids, policy ids, run ids, signers, timestamps, and redactions.
Pass: export completeness is measurable.

### Story ERP-HS-020 - SAP Cutover Cockpit
As Marcus, I want a migration cutover cockpit so that each module can switch when parity proof is green.
Pass: cockpit shows extract, transform, load, reconcile, policy-bind, and rollback status.
Pass: cutover cannot start with unresolved P1 reconciliation defects.

## Surface Map

### Surface ERP-SURF-01 - Coverage Command Center
```
+-------------------------------------------------------------+
| ERP Coverage                                                |
| Entity: Global group  Period: 2026-05  Pack: KR+EU+US       |
| [Close] [Procure] [Inventory] [Manufacture] [Trade] [Audit] |
| Risk: 3 blockers  Migration: 92% mapped  Evidence: 99.7%    |
+-------------------------------------------------------------+
```

### Surface ERP-SURF-02 - Module Parity Matrix
```
+ Module + SAP peer + Oyatie owners + Coverage + Gaps + Gate +
| FI     | S/4 FI   | payments, treasury, finops | 0.91 | 2 | yellow |
| MM     | S/4 MM   | marketplace, warehouse      | 0.84 | 5 | red    |
+-------------------------------------------------------------+
```

### Surface ERP-SURF-03 - Migration Dry Run
```
+ Extract + Transform + Load + Reconcile + Cedar + Rollback +
| FI 2.4M rows | 0.03% reject | lineage 100% | ready |
+-------------------------------------------------------------+
```

### Surface ERP-SURF-04 - Close Console
```
+ Entity + Phase + Owner + Blocker + Evidence + Timer +
| KR01 | accruals | Helena | tax pack missing | sealed 88% | 03:14 |
+-------------------------------------------------------------+
```

### Surface ERP-SURF-05 - Req-to-Pay Workbench
```
+ Request + Supplier + Budget + KYC + PO + GR + Invoice + Pay +
| REQ-42 | S-900 | held | green | issued | partial | blocked |
+-------------------------------------------------------------+
```

### Surface ERP-SURF-06 - Plant Operations Board
```
+ Order + Work center + Material + Quality + Inventory + Ship +
| MO-88 | line 4 | MAT-11 | hold | reserved | blocked |
+-------------------------------------------------------------+
```

### Surface ERP-SURF-07 - Treasury Cockpit
```
+ Cash + AR + AP + Payroll + Tax + Debt + FX + Alert +
| 92d runway | +4.2M | -3.1M | -1.0M | -0.7M | -0.4M | KRW/USD |
+-------------------------------------------------------------+
```

### Surface ERP-SURF-08 - Audit Evidence Graph
```
+ Control + Journal + Workflow + Cedar + Event + Export +
| SOX-P2P-17 | JE-219 | RUN-32 | DEC-7 | EVT-11 | ready |
+-------------------------------------------------------------+
```

## Data Model

### Entity ERP-ENT-01 - ErpModuleCoverage
- Fields: coverage_id, module_code, sap_peer, owner_products, owner_microservices, coverage_score, gap_count, gate_status.
- Relationships: owns many RequirementRows, references many MigrationPlaybooks.
- Invariant: coverage_score cannot be green while any P1 gap is open.

### Entity ERP-ENT-02 - SourceSystem
- Fields: source_system_id, vendor, version, tenant_scope, region, extract_method, credentials_ref, retirement_date.
- Relationships: source for many SourceObjects and MigrationRuns.
- Invariant: credentials_ref is a secret reference, never a stored credential.

### Entity ERP-ENT-03 - SourceObject
- Fields: source_object_id, source_system_id, source_type, source_pk, checksum, data_class, owner_module.
- Relationships: maps to one or more OyatieObjects.
- Invariant: source_pk is immutable after first extract.

### Entity ERP-ENT-04 - OyatieObjectMapping
- Fields: mapping_id, source_object_id, target_microservice, target_object_type, target_object_id, confidence, human_review_state.
- Relationships: belongs to MigrationRun; produces AuditEvidence.
- Invariant: confidence below threshold requires human review before cutover.

### Entity ERP-ENT-05 - AccountingDocument
- Fields: doc_id, entity_id, period, currency, ledger, source_ref, posting_status, amount_dr, amount_cr.
- Relationships: has many JournalLines; references WorkflowRun.
- Invariant: debits equal credits per ledger and currency.

### Entity ERP-ENT-06 - JournalLine
- Fields: line_id, doc_id, account, cost_center, profit_center, project_id, amount, tax_code, source_object_ref.
- Relationships: belongs to AccountingDocument; links to OntologyObject.
- Invariant: every posted line has source_object_ref.

### Entity ERP-ENT-07 - SupplierProfile
- Fields: supplier_id, legal_name, tax_id, bank_ref, risk_rating, sanctions_state, kyc_state, pack_set.
- Relationships: owns PurchaseOrders, Contracts, Invoices.
- Invariant: active supplier requires green KYC and current sanctions screen.

### Entity ERP-ENT-08 - PurchaseRequest
- Fields: request_id, requester, entity, cost_center, item_refs, budget_hold_id, approval_state, urgency.
- Relationships: creates PurchaseOrder after approval.
- Invariant: spend above threshold requires approver distinct from requester.

### Entity ERP-ENT-09 - GoodsReceipt
- Fields: receipt_id, po_id, dock_id, received_by, quantity, discrepancy_state, photo_evidence_ref.
- Relationships: matches Invoice and PurchaseOrder.
- Invariant: discrepancy_state unresolved blocks invoice auto-match.

### Entity ERP-ENT-10 - QualityHold
- Fields: hold_id, material_id, lot_id, plant_id, reason_code, severity, disposition_state, release_signature.
- Relationships: blocks Shipments and InvoiceRelease.
- Invariant: release_signature required for release.

### Entity ERP-ENT-11 - ProductionOrder
- Fields: order_id, bom_id, routing_id, plant_id, planned_qty, actual_qty, status, cost_collector_id.
- Relationships: consumes InventoryLots and emits ProductionActuals.
- Invariant: completion requires material backflush or explicit variance reason.

### Entity ERP-ENT-12 - TreasuryForecast
- Fields: forecast_id, horizon_days, base_currency, inflow_total, outflow_total, confidence_band, drift_score.
- Relationships: reads AP, AR, Payroll, Tax, Debt, FX objects.
- Invariant: drift_score above threshold opens investigation workflow.

## Cedar Policy Model

- Principal erp::Requester can create PurchaseRequest within tenant and cost_center scope.
- Principal erp::Approver can approve PurchaseRequest only when not the requester.
- Principal erp::ReceivingClerk can create GoodsReceipt but cannot approve payment.
- Principal erp::PaymentReleaser can release payment only after ThreeWayMatch green.
- Principal erp::Controller can post AccountingDocument after period open check.
- Principal erp::Auditor can read evidence export but not mutate operational objects.
- Principal erp::TradeOfficer can release export shipment after sanctions and license checks pass.
- Principal erp::QualityManager can release QualityHold only with disposition_state approved.
- Action erp::post_journal requires balanced document and open period.
- Action erp::approve_spend requires budget_hold active and segregation-of-duties guard.
- Action erp::release_shipment requires no active quality hold and trade screen current.
- Action erp::start_cutover requires all P1 reconciliation defects closed.
- Resource erp::AccountingDocument includes tenant_id, entity_id, ledger, period, data_class.
- Resource erp::SupplierProfile includes tenant_id, country, risk_rating, pack_set.
- Resource erp::QualityHold includes tenant_id, plant_id, severity, release_signature.

## Workflow Engine Integration

- Node ERP-WF-01 ExtractSourceObjects: reads SourceSystem and emits source batch.
- Node ERP-WF-02 TransformToOyatieObjects: maps vendor objects to ontology types.
- Node ERP-WF-03 ReconcileCounts: compares source count, target count, amount totals, and checksums.
- Node ERP-WF-04 BindCedarPolicies: attaches tenant, role, and pack policy fragments.
- Node ERP-WF-05 DryRunCutover: simulates module switch without mutating source.
- Node ERP-WF-06 ApproveCutover: requires Marcus plus module owner approval.
- Node ERP-WF-07 ExecuteCutover: freezes source, imports delta, seals lineage.
- Node ERP-WF-08 RollbackCutover: reopens source and invalidates target writes by run id.
- Node ERP-WF-09 ReqToPayStart: creates request and budget hold.
- Node ERP-WF-10 SupplierRiskCheck: runs KYC, tax, sanctions, bank verification.
- Node ERP-WF-11 ApproveSpend: routes approval by amount, entity, and pack.
- Node ERP-WF-12 IssuePurchaseOrder: creates PO and supplier commitment.
- Node ERP-WF-13 ReceiveGoods: records dock receipt and discrepancy state.
- Node ERP-WF-14 MatchInvoice: performs two-way or three-way match.
- Node ERP-WF-15 ReleasePayment: emits payment command only after match green.
- Node ERP-WF-16 QualityHoldPlaced: blocks shipment and billing.
- Node ERP-WF-17 QualityDisposition: records scrap, rework, release, or supplier return.
- Node ERP-WF-18 CloseChecklistStep: seals close task evidence.
- Node ERP-WF-19 ForecastLiquidity: computes cash forecast and drift.
- Node ERP-WF-20 AuditEvidenceExport: packages event ids and policy decisions for auditors.

## AI / Intelligence Integration

- ADR-0220 layer: Intelligence classifies migration reject reasons without mutating records.
- ADR-0220 layer: Intelligence ranks reconciliation defects by financial materiality.
- ADR-0255 layer 1: tenant-private retrieval reads migration lineage, source docs, policy receipts, and workflow runs.
- ADR-0255 layer 1: tenant-private retrieval redacts PII by pack before explanation.
- ADR-0255 layer 2: cross-tenant aggregate learns common SAP mapping patterns without exposing tenant data.
- ADR-0255 layer 2: aggregate recommendations are suggestions until human approval.
- Capability erp.migration.map-object suggests target ontology object.
- Capability erp.close.explain-blocker summarizes close blockers with cited evidence.
- Capability erp.procurement.supplier-risk explains risk flags from KYC and sanctions.
- Capability erp.treasury.forecast-drift explains why forecast changed.
- Capability erp.quality.disposition-assist proposes rework or scrap paths with citations.
- Prohibited: Intelligence cannot post journals, release payments, or override Cedar denial.

## Pack Overlays

- Pack KR-CSAP activates strict region, Korean cryptographic-module-validation key custody, Korean tax invoice rules, and labor overlay.
- Pack EU-DORA activates financial resilience evidence, EU data residency, and vendor exit proof.
- Pack BR-LGPD activates tax-id validation, consent redaction, and LGPD data-subject response hooks.
- Pack US-SOX activates SOX control evidence, segregation-of-duties, and audit export retention.
- Pack JP-ISMAP activates JP residency, invoice preservation, and local audit-language pack.
- Pack Healthcare activates HIPAA class redaction and patient-billing separation.
- Pack Public-Sector activates procurement transparency, conflict-of-interest disclosure, and retention floor.

## SLO Targets


## Telemetry

- EVT-ERP-MODULE-COVERAGE-SCORED emits module_code, coverage_score, gap_count, and gate_status.
- EVT-ERP-SOURCE-EXTRACT-STARTED emits source_system_id, batch_id, module_code, and checksum.
- EVT-ERP-SOURCE-EXTRACT-COMPLETED emits row_count, reject_count, checksum, and duration_ms.
- EVT-ERP-OBJECT-MAPPED emits source_object_id, target_object_id, confidence, and reviewer.
- EVT-ERP-RECONCILIATION-FAILED emits defect_id, materiality, module_code, and owner.
- EVT-ERP-CUTOVER-APPROVED emits approver, module_code, source_system_id, and rollback_ref.
- EVT-ERP-CUTOVER-EXECUTED emits cutover_id, freeze_time, delta_count, and evidence_chain_id.
- EVT-ERP-CUTOVER-ROLLED-BACK emits cutover_id, reason, rollback_checkpoint, and operator.
- EVT-ERP-JOURNAL-POSTED emits doc_id, ledger, period, amount_total, and policy_decision_id.
- EVT-ERP-SUPPLIER-KYC-COMPLETED emits supplier_id, kyc_state, risk_rating, and pack_set.
- EVT-ERP-BUDGET-HOLD-CREATED emits request_id, budget_hold_id, amount, and currency.
- EVT-ERP-THREE-WAY-MATCH-FAILED emits po_id, invoice_id, receipt_id, and mismatch_reason.
- EVT-ERP-PAYMENT-RELEASED emits payment_id, supplier_id, amount, and approver_chain.
- EVT-ERP-QUALITY-HOLD-PLACED emits hold_id, lot_id, plant_id, and severity.
- EVT-ERP-QUALITY-HOLD-RELEASED emits hold_id, disposition, signer, and evidence_ref.
- EVT-ERP-CLOSE-STEP-SEALED emits close_id, step_id, owner, and audit_event_id.
- EVT-ERP-CASH-FORECAST-DRIFT emits forecast_id, drift_score, top_driver, and threshold.
- EVT-ERP-AUDIT-EXPORT-GENERATED emits export_id, period, controls, redaction_pack, and requester.

## Migration Playbook Index

- SAP S/4HANA: FI/CO, MM, SD, PP, QM, PM, HCM, PS, PLM, GTS, TM, EWM, TRM, RE-FX.
- SAP ECC: legacy account groups, custom Z tables, ABAP exits, batch-input recordings.
- Oracle Fusion Cloud ERP: ledger, procurement, projects, inventory, and HCM objects.
- Oracle E-Business Suite: GL, AP, AR, PO, INV, WIP, OM, HRMS extracts.
- NetSuite: subsidiary, chart, item, sales order, purchase order, invoice, customer.
- Microsoft Dynamics 365: finance, supply chain, commerce, project operations, HR.
- Workday: worker, position, payroll, benefits, talent, supplier invoice objects.
- Infor: manufacturing, warehouse, asset, and industry-specific data objects.
- IFS: asset-intensive manufacturing, maintenance, projects, and service objects.
- Sage Intacct: GL, AP, AR, dimensions, cash management, projects.
- QuickBooks: SMB chart, customer, vendor, item, invoice, bill, payment migration.
- Odoo: modular ERP tenant migration with explicit app inventory.

## Capability Tier Deltas


## Competitive Positioning

- SAP S/4HANA: oyatie wins on tenant portability, Cedar policy, workflow evidence, and no ABAP lock-in.
- Oracle Fusion: oyatie wins on open workflow specs, regional pack overlay, and audit-chain native design.
- Microsoft Dynamics 365: oyatie wins on universal tenant graph and policy-gated cross-product automation.
- NetSuite: oyatie wins on growth path from SMB to enterprise without replatforming.
- Workday: oyatie wins on HR-to-finance-to-workplace continuity under one identity.
- Infor: oyatie wins on composable industry overlays and unified migration proof.
- IFS: oyatie wins when asset, project, field, and finance data need one policy graph.
- Sage Intacct: oyatie wins by connecting finance to inventory, procurement, and workflow from day one.
- Odoo: oyatie wins on governed extension, audit chain, and regulated tenant controls.

## Roadmap

- Wave 1: coverage cockpit, module matrix, migration dry run, FI/CO skeleton, audit event contract.
- Wave 2: procurement, AP, AR, supplier onboarding, three-way match, warehouse receiving.
- Wave 3: manufacturing, quality, production costing, plant maintenance, project systems.
- Wave 4: treasury, cash forecast, FX exposure, intercompany, consolidation.
- Wave 5: trade, customs, sanctions, transportation, EWM, public-sector procurement.
- Phase M04: internal oyatie tenant dogfood for finance and procurement workflows.
- Phase M05: design-partner enterprise migration dry runs.

## Cross-Product Dependencies

- marketplace owns supplier directory, DealSet, vendor contract, and catalog procurement handoff.
- payments owns payment initiation, settlement, reconciliation, and chargeback handoff.
- finops-portal owns cost center analytics, budget variance, and chargeback views.
- treasury owns cash forecast, debt, FX, liquidity, and bank account risk.
- warehouse owns receiving, picking, lot, serial, dock, and inventory reservation.
- supply-chain-planning owns demand, supply network, MRP, and ATP handoff.
- workflow-engine owns every durable ERP saga.
- workflow-studio owns tenant-editable process templates.
- ontology owns business object types and relationships.
- policy-engine owns Cedar permit decisions.
- audit-chain owns immutable evidence and export.
- intelligence owns explanation, mapping suggestions, and forecast-drift analysis.

## Failure Modes + Recovery

- Failure: migration extract checksum mismatch. Recovery: halt import, keep source read-only, rerun extract with previous checkpoint.
- Failure: source object maps to multiple target objects. Recovery: require human review and preserve mapping candidates.
- Failure: journal import unbalanced. Recovery: reject document, emit reconciliation defect, and block cutover.
- Failure: Cedar policy denies expected approver. Recovery: route to policy exception workflow, do not bypass.
- Failure: three-way match false positive. Recovery: quarantine invoice, reopen match, and notify AP lead.
- Failure: supplier bank verification stale. Recovery: block payment and rerun bank verification.
- Failure: quality hold not propagated to shipment. Recovery: cancel shipment release and run hold-propagation replay.
- Failure: inventory reservation crosses illegal residency. Recovery: release reservation and reroute to compatible cell.
- Failure: treasury forecast missing bank feed. Recovery: degrade confidence band and open feed incident.
- Failure: close evidence export missing event id. Recovery: reseal from audit-chain and mark close step yellow.
- Failure: intercompany side missing. Recovery: create confirmation task for counterparty entity.
- Failure: workflow-engine retry duplicates PO. Recovery: idempotency key collapses duplicate and emits duplicate-attempt event.
- Failure: AI mapping hallucination. Recovery: suggestions are non-mutating and require reviewer approval.
- Failure: regional pack conflict. Recovery: policy-engine conflict resolver chooses stricter rule and opens legal review.

## AI substrate + Cellular automation

This product consumes current SSOT doctrine for the intelligence substrate, cellular automation, and cloud-native delivery:

- D-CICD-AUTHORITY binds this lane to the branch-protected `presubmit` presubmit gate as live merge authority; local command output is transition evidence only. Historical ADR-0346 verifier wording is retained only where it does not conflict with `registry/stores/design-store.json` current truth.
- D-GOVERNANCE-CENTRAL: central PaC/CaC/PDP/evidence pipelines own governance authority; do not scatter authority across local CLI lanes.
- ADR-0348 binds ERP module placement, migration runs, supply-chain workflows, and regulated evidence export to cellular topology. Enforcement evidence flows through central governance and the branch-protected `presubmit` gate, not scattered local lanes.
- D-CICD-AUTHORITY keeps one canonical CI authority now (`presubmit`) and the owned ci cutover later; self-hostable delivery references are subordinate to the current SSOT and are not parallel merge authorities. Historical ADR-0349 substrate wording is retained only as non-authoritative context until reconciled with the current stores.

## References

- docs/standards/documentation-rigor.md
- docs/personas/MASTER-ROSTER-2026-05-21.md
- docs/decisions/ADR-0702-identity-authz-live-apex.md
- docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- docs/decisions/ADR-0708-platform-foundations-live-apex.md
- docs/adr-archive/ADR-0255-intelligence-as-two-layer-ai-substrate.md
- docs/adr-archive/ADR-0263-observability-emission-contract.md
- docs/decisions/ADR-0705-product-protocol-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/adr-archive/ADR-0316-capability-tier-over-product-fragmentation.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- specs/microservices/accounting.json
- specs/microservices/hr.json
- specs/microservices/payroll.json
- docs/products/workplace-integration/PRD.md

## 2a. Acceptance criteria traceability (required)

This section is a planning-maturity contract only. It does **not** claim runtime, product-ready, or hyperscaler-ready status; promotion still requires fresh CI, SLO, security, SBOM, rollback/DR, owner/RACI, and product-pain evidence.

| AC-ID | Given | When | Then | Test ID | Test path |
|---|---|---|---|---|---|
| ERP-PRD-AC-001 | The ERP Coverage PRD is used as a planning contract and finance ledger, procurement, payroll, supply-chain, and SAP-displacement module contracts are referenced by a promotion packet | The planned-maturity gate scans product PRDs | ERP module acceptance is linked to test and evidence paths instead of generic prose | ERP-PRD-GATE-001 | `cloud/cloud-ci/gates/pipeline-planned-maturity-app/tests/planned_maturity.rs::live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated` |
| ERP-PRD-AC-002 | ERP module-pack readiness is evaluated | Readiness evidence is evaluated | fresh ledger/procurement/payroll/workflow/audit evidence and user-pain validation is required outside this PRD | ERP-PRD-GATE-002 | `cloud/cloud-ci/gates/pipeline-planned-maturity-app/tests/planned_maturity.rs::live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated` |

## 9b. Verification commands (required) — one runnable check per metric

| Metric | Verification command | Pass criterion | CI lane |
|---|---|---|---|
| ERP ledger/module/workflow planning maturity | `buck2 test //cloud/cloud-ci/gates/pipeline-planned-maturity-app:pipeline-planned-maturity-app-gate` | At least one ERP row names ledger, procurement/payroll, workflow, SAP-parity, and audit obligations | `presubmit` |
| ERP product-ready non-claim boundary | `buck2 test //cloud/cloud-ci/gates/pipeline-planned-maturity-app:pipeline-planned-maturity-app-gate` | An ERP promotion packet cannot treat this PRD as product-ready evidence without fresh CI and product-pain proof | `presubmit` |
