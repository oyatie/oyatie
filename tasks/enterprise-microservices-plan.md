# RETIRED — pointer hub only

This file is retired per the markdown-retirement-policy (`/specs/markdown-retirement-policy.json`).
The worktree `agent/enterprise-microservices-20260523T070244Z` has been pruned (content merged to dev).

Authoritative planning SSOT: `/specs/masterplan.json`
Microservice specs: `/specs/microservices/`
ADR decisions: `docs/decisions/`

## Archived content (non-authoritative)

Authoritative inputs:

1. `specs/microservices/hr.json`, `specs/microservices/payroll.json`, and `specs/microservices/accounting.json`.
2. `docs/decisions/ADR-0709-general-live-apex.md`.
3. `docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md`.
4. `docs/AGENTS.md` done-definition, Oya VCS, and CI mirror requirements.

## Best-practice research handoff

Official/upstream sources checked on 2026-05-23:

- IRS Publication 15 (2026): payroll engines must treat withholding, Social Security/Medicare, FUTA, W-2/1099 thresholds, supplemental wages, and payroll outsourcing duties as versioned annual rulepack inputs. Source: https://www.irs.gov/publications/p15
- IRS Publication 15-T (2026): U.S. federal income tax withholding methods are a separate annually versioned calculation source. Source: https://www.irs.gov/publications/p15t
- U.S. DOL FLSA recordkeeping: HR/attendance must retain wages paid, daily hours, and workweek hours so payroll derivation is evidence-backed. Source: https://www.dol.gov/agencies/whd/fact-sheets/21-flsa-recordkeeping
- EEOC employment tests guidance and AI initiative: recruiting/performance automation must preserve adverse-impact and disability-accommodation evidence instead of making AI-only decisions. Sources: https://www.eeoc.gov/laws/guidance/employment-tests-and-selection-procedures and https://www.eeoc.gov/ai
- FASB standards page: U.S. GAAP authority resolves through the FASB Accounting Standards Codification, so accounting mappings must be rulepack/source-versioned. Source: https://www.fasb.org/standards
- IFRS IAS 1: IFRS compliance claims require complete financial statement presentation and explicit compliance, so accounting exports cannot claim IFRS without evidence. Source: https://www.ifrs.org/issued-standards/list-of-standards/ias-1-presentation-of-financial-statements.html/
- SEC ICFR guidance: financial posting and close workflows need risk-based internal-control evidence, not mutable spreadsheet close. Source: https://www.sec.gov/rule-release/33-8212
- Korea Labor Standards Act / MOEL: Korea HR rulepacks must model legal working conditions, and Article 93 rules-of-employment reporting activates at ten or more employees. Sources: https://moel.go.kr/english/policy/laborStandards.do and https://law.go.kr/LSW/lsInfoP.do?lsiSeq=199151&urlMode=engLsInfoR&viewCls=engLsInfoR
- OpenAPI Initiative specification site checked 2026-05-23: OpenAPI is the formal vendor-neutral description standard for HTTP APIs, and the latest published rendering is OpenAPI 3.2.0. Sources: https://www.openapis.org/ and https://spec.openapis.org/oas/latest
- SAP Ariba Strategic Sourcing Suite documentation: source-to-contract/source-to-pay parity requires supplier lifecycle/performance, sourcing, contracts, and ERP-generated purchase orders from contracted terms. Source: https://help.sap.com/docs/strategic-sourcing/sap-ariba-product-sourcing/sap-ariba-strategic-sourcing-suite?locale=en-US
- Oracle Procurement Cloud official page: source-to-settle procurement suites cover procure-to-pay, strategic sourcing, supplier management, requisitions, purchase orders, invoices, and supplier/item/invoice data for cloud ERP. Source: https://www.oracle.com/erp/procurement/
- Coupa supplier purchase-order documentation: supplier-facing procure-to-pay needs purchase-order receipt channels and invoice creation against purchase orders, which supports PO/invoice matching design. Source: https://docs.coupa.com/en/supplier-documentation/coupa-for-suppliers/the-coupa-supplier-portal-or-csp/features-and-processes-in-the-coupa-supplier-portal/purchase-orders/about-purchase-orders
- SAP S/4HANA Cloud Cash and Liquidity Management documentation: treasury parity requires bank account master data, cash position display, cash-flow analyzer, liquidity trend/forecast analysis, and integration from bank statements, accounting, TRM, MM, and SD via cash-flow sources. Sources: https://help.sap.com/docs/SAP_S4HANA_CLOUD/186460fdc35a4b64a713da9bb00deb1e/c7999f525c5b224fe10000000a445394.html and https://help.sap.com/docs/SAP_S4HANA_CLOUD/ac319d8fa4ea4624b40a58d23e3c4627/b99f6156f5b61d58e10000000a4450e5.html
- Oracle Fusion Cloud Cash Management documentation: cash positioning projects daily cash needs by bank account, currency, legal entity, source, balances, and transactions, while dashboard views expose cash balance, bank-account deficit/target-balance risk, five-day forecast, unreconciled items, and missing statements. Sources: https://docs.oracle.com/en/cloud/saas/financials/25d/faofc/cash-positioning.html and https://docs.oracle.com/en/cloud/saas/financials/25c/faofc/overview-of-the-cash-management-dashboard.html
- SAP S/4HANA Cloud Warehouse Management / EWM documentation: warehouse parity requires inbound/outbound processing, physical inventory and counting, and warehouse operations with goods receipt, putaway, picking, packing, and shipping boundaries. Sources: https://help.sap.com/docs/SAP_S4HANA_CLOUD/87f9b54f9c4f4e75aff0061860a6589a/b8e1242b86f5403f9c81a65e9a188f10.html and https://help.sap.com/docs/SAP_S4HANA_CLOUD/87f9b54f9c4f4e75aff0061860a6589a/4938549c790d4e2cbeade042b59becd3.html
- Oracle Warehouse Management Cloud documentation: WMS parity requires inbound receiving/receipts, putaway/capacity-aware inventory, outbound allocation/picking, cycle counts, and explicit activity parameters without implying this slice runs the WMS. Sources: https://docs.oracle.com/en/cloud/saas/warehouse-management/26a/owmol/overview.html and https://docs.oracle.com/en/cloud/saas/warehouse-management/26b/owmwr/allocation.html
- Microsoft Dynamics 365 Supply Chain Management warehouse management overview: warehouse systems cover inbound, outbound, inventory movement, picking, putaway, replenishment, and physical inventory workflows. Source: https://learn.microsoft.com/en-us/dynamics365/supply-chain/warehousing/warehouse-management-overview
- SAP S/4HANA Cloud Production Planning and Control documentation: PP parity requires materials availability planning, MRP simulations, planned/production orders, BOMs, routings, work centers, and capacity/scheduling evidence without implying this branch runs MRP or shop-floor execution. Sources: https://help.sap.com/docs/SAP_S4HANA_CLOUD/2bba750d1e124e1ea2a039bb1cd9b6c5?locale=en-US&state=PRODUCTION&version=2602.500 and https://help.sap.com/docs/SAP_S4HANA_CLOUD/2bba750d1e124e1ea2a039bb1cd9b6c5/e8c6bd131b9d471698716f65e5852e97.html
- Oracle Fusion Cloud Manufacturing documentation: production process design uses work definitions for production processes, and work orders carry item, quantity, status, dates, work definition, material availability, and WMS movement boundaries. Sources: https://docs.oracle.com/en/cloud/saas/supply-chain-and-manufacturing/24b/faumf/overview-of-production-process-design.html and https://docs.oracle.com/en/cloud/saas/supply-chain-and-manufacturing/25c/faumf/overview-of-work-orders.html
- Microsoft Dynamics 365 Supply Chain production-control documentation: BOMs define components, routes define operations/resources/times, and production orders are based on BOMs and routes. Sources: https://learn.microsoft.com/en-us/dynamics365/supply-chain/production-control/bill-of-material-bom, https://learn.microsoft.com/en-us/dynamics365/supply-chain/production-control/routes-operations, and https://learn.microsoft.com/en-us/dynamics365/supply-chain/production-control/production-process-overview

## Parallelizable ChangeSet sequence

### CS-ENT-HR-001 — HR employment domain foundation

Path envelope:

- `microservices/hr/README.md`
- `microservices/hr/PRD.md`
- `microservices/hr/crates/oya-hr-employment-domain/**`
- `microservices/hr/catalog/oya-hr-employment-domain.yaml`
- `registry/catalog/oya-hr-employment-domain.yaml`
- `specs/microservices/hr.json`
- `Cargo.toml`
- `tasks/enterprise-microservices-*`

Acceptance criteria:

- Employee records require tenant, legal entity, person ref, employment status, data class, schema version, and audit evidence.
- Employee lifecycle events are tenant/legal-entity scoped and idempotency-keyed.
- Identifiers and evidence refs reject prefix-only, whitespace/control, traversal, and token/secret-shaped values.
- Korea rulepack threshold evaluation opens rules-of-employment obligations at 10 employees.
- Korea labor-management council evidence requirements activate at 30 employees.
- Labor-compliance obligations carry stable obligation identity, rulepack effective date, state, and idempotency key for downstream Workflow wiring.
- Domain layer remains pure: no storage, network, clock, workflow dispatch, payroll derivation, or regulator filing I/O.

Verification:

- `cargo test -p oya-hr-employment-domain`
- `cargo clippy -p oya-hr-employment-domain --all-targets -- -D warnings`
- `./bin/oya gate validate cargo-prefix --workspace Cargo.toml --prefix oya-`

### CS-ENT-PAYROLL-001 — Payroll run and wage-ledger foundation

Disjoint path envelope:

- `microservices/payroll/README.md`
- `microservices/payroll/PRD.md`
- `microservices/payroll/crates/oya-payroll-run-domain/**`
- `microservices/payroll/catalog/oya-payroll-run-domain.yaml`
- `registry/catalog/oya-payroll-run-domain.yaml`
- `specs/microservices/payroll.json`
- `Cargo.toml`

Acceptance focus: payee abstraction, trial close before production close, append-only wage ledger entries, rulepack effective dates, metadata-only evidence digests, statutory export evidence, balanced accounting draft, and rollback-first close promotion.

Verification:

- `cargo test -p oya-payroll-run-domain`
- `cargo clippy -p oya-payroll-run-domain --all-targets -- -D warnings`
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-payroll-run-domain-foundation-1779522600.json --severity error`

### CS-ENT-PAYROLL-002 — Payroll HR leave-impact intake metadata foundation

Disjoint path envelope:

- `microservices/payroll/crates/oya-payroll-run-domain/src/lib.rs`
- `microservices/payroll/crates/oya-payroll-run-domain/tests/hr_leave.rs`
- `microservices/payroll/crates/oya-payroll-run-app/src/lib.rs`
- `microservices/payroll/crates/oya-payroll-run-app/tests/hr_leave.rs`
- `microservices/payroll/crates/oya-payroll-run-api/src/lib.rs`
- `microservices/payroll/crates/oya-payroll-run-api/tests/contracts.rs`
- `microservices/payroll/contracts/openapi-v1.yaml`
- `microservices/payroll/contracts/openapi-v1.meta.yaml`
- `specs/microservices/payroll.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-payroll-hr-leave-impact-intake-1779535200.json`
- `evidence/vcs/cs-ent-payroll-hr-leave-impact-intake-oya-vcs-lifecycle-20260523.json`

Acceptance focus: Payroll AC-10 metadata-only intake for HR AC-04 leave or
attendance payroll-impact handoffs. The domain requires the canonical HR source
topic, source HR idempotency key, payroll period, payee/employee/leave IDs,
rulepack basis, decision/routing/payroll-impact evidence, and payroll-owned
intake evidence. The app crate emits a metadata-only payroll-owned integration
envelope, and the API/OpenAPI layers expose preview DTO/contract shapes for
later cloud adapters. This slice makes no payroll calculation, leave balance
calculation, HR service call, storage, Workflow execution, audit-chain runtime
emission, deployed HTTP endpoint, or cloud runtime claim.

Verification:

- `cargo test --locked -p oya-payroll-run-domain -p oya-payroll-run-app -p oya-payroll-run-api`
- `cargo clippy --locked -p oya-payroll-run-domain -p oya-payroll-run-app -p oya-payroll-run-api --all-targets -- -D warnings`
- `./bin/oya gate validate api-semver --contracts-dir microservices/payroll/contracts`
- `cargo fmt --all -- --check`
- enterprise package-group regression tests/clippy across HR, Payroll, Accounting, and Tenant RBAC crates
- data-class annotation scan for touched Payroll public fields and OpenAPI `x-data-class` fields
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-payroll-hr-leave-impact-intake-1779535200.json --severity error`


### CS-ENT-PAYROLL-003 — Payroll HTTP runtime adapter foundation

- **Intent:** Bind existing payroll API DTO/OpenAPI preview routes to the repo-native Hyper router/middleware foundation without adding a deployed listener or persistence.
- **Primary scope:** `microservices/payroll/crates/oya-payroll-run-runtime`, `Cargo.toml`, `registry/catalog/oya-payroll-run-runtime.yaml`, payroll spec AC-11/D-10, payroll OpenAPI metadata, evidence/audit closeout.
- **Acceptance:** Runtime tests dispatch trial-close, accounting journal draft, HR leave-impact intake, invalid JSON/domain errors, route manifest, bounded server config, and health non-claims through transport-neutral `HttpRequest`/`HttpResponse`.
- **Non-claims:** no live listener deployment, storage, Workflow dispatch, statutory filing rails, HR/accounting network calls, disbursement rails, payroll calculation, or runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-payroll-run-runtime`; `cargo clippy --locked -p oya-payroll-run-runtime --all-targets -- -D warnings`; `cargo fmt --all -- --check`; payroll OpenAPI semver; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.


### CS-ENT-PAYROLL-004 — Payroll in-memory storage seam reference

- **Intent:** Add a volatile in-memory payroll storage port/adaptor reference for trial-close audit, accounting dispatch, and HR leave-impact intake metadata without durable storage or runtime side effects.
- **Primary scope:** `microservices/payroll/crates/oya-payroll-run-storage-adapter-inmemory`, `Cargo.toml`, `registry/catalog/oya-payroll-run-storage-adapter-inmemory.yaml`, payroll spec AC-12/D-11, evidence/audit closeout.
- **Acceptance:** Storage tests persist all payroll metadata record kinds, preserve topic/tenant/legal-entity/run/primary refs, evidence counts, payload data-class labels, and idempotency keys, reject duplicate or unsafe idempotency keys, and expose capability flags that keep durable backend, Postgres/RLS, payroll calculation, statutory filing rails, disbursement rails, Workflow dispatch, HR/accounting calls, and audit-chain emission false.
- **Non-claims:** volatile test/local reference only; no durable backend, Postgres/RLS, payroll calculation, statutory filing transport, disbursement rails, Workflow dispatch, HR/accounting network call, cloud integration, or runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-payroll-run-storage-adapter-inmemory`; `cargo clippy --locked -p oya-payroll-run-storage-adapter-inmemory --all-targets -- -D warnings`; payroll package-group tests/clippy; enterprise package-group tests/clippy; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.





### CS-ENT-PROCUREMENT-001 — Procurement source-to-pay domain foundation

- **Intent:** Add a flat procurement source-to-pay domain foundation for supplier qualification, requisition approval, purchase order issuance, and three-way matching so ERP/SAP MM/SRM parity has a repo-native service destination before cloud integration.
- **Primary scope:** `microservices/procurement/crates/oya-procurement-source-to-pay-domain`, `specs/microservices/procurement.json`, `registry/catalog/oya-procurement-source-to-pay-domain.yaml`, `Cargo.toml`, Tenant RBAC ERP parity map MM/SRM rows, evidence/audit closeout.
- **Acceptance:** Domain tests validate supplier KYB/risk/vendor-master evidence, purchase requisition approval with qualified supplier and budget/policy/approval evidence, purchase order issuance from approved requisition, three-way PO/receipt/invoice quantity and amount match, unsafe source/evidence ref refusal, and explicit false payment/inventory/supplier-network/cloud runtime capability flags.
- **Non-claims:** no durable persistence, no supplier portal/network call, no Workflow execution, no inventory mutation, no payment execution, no statutory filing, no cloud deployment, no production procurement parity claim, and no runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-procurement-source-to-pay-domain`; `cargo clippy --locked -p oya-procurement-source-to-pay-domain --all-targets -- -D warnings`; ERP parity map tests/clippy; enterprise package-group tests/clippy including procurement; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-TREASURY-001 — Treasury cash-position domain foundation

- **Intent:** Add a flat treasury cash-management domain foundation for bank-account approval, cash-position snapshots, liquidity forecasts, and cash-transfer proposal metadata so ERP/SAP FI/TRM parity has a repo-native service destination before cloud integration.
- **Primary scope:** `microservices/treasury/crates/oya-treasury-cash-domain`, `specs/microservices/treasury.json`, `registry/catalog/oya-treasury-cash-domain.yaml`, `Cargo.toml`, Tenant RBAC ERP parity map FI/TRM rows, evidence/audit closeout.
- **Acceptance:** Domain tests validate bank-account approval with target balance and control evidence, cash-position closing-balance derivation from bank statement/exposure-flow evidence, liquidity forecast shortfall/breach calculation, cash-transfer proposal surplus/need checks, unsafe source/evidence ref refusal, invalid date/currency refusal, and explicit false payment/bank-network/cloud/runtime-audit capability flags.
- **Non-claims:** no durable persistence, no live bank connectivity or bank-network call, no payment execution, no accounting ledger mutation, no Workflow execution, no statutory filing, no cloud deployment, no production treasury parity claim, and no runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-treasury-cash-domain`; `cargo clippy --locked -p oya-treasury-cash-domain --all-targets -- -D warnings`; ERP parity map tests/clippy; enterprise package-group tests/clippy including treasury/procurement; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-WAREHOUSE-001 — Warehouse inventory domain foundation

- **Intent:** Add a flat warehouse inventory domain foundation for goods receipt, putaway stock positioning, inventory reservation, pick confirmation, and cycle-count reconciliation so ERP/SAP MM/EWM parity has a repo-native service destination before cloud integration.
- **Primary scope:** `microservices/warehouse/crates/oya-warehouse-inventory-domain`, `specs/microservices/warehouse.json`, `registry/catalog/oya-warehouse-inventory-domain.yaml`, `Cargo.toml`, Tenant RBAC ERP parity map MM/EWM rows, evidence/audit closeout.
- **Acceptance:** Domain tests validate goods receipt with PO/inbound-load evidence and explicit false procurement/accounting/durable-write flags, putaway into capacity-checked bins, reservation without over-allocation, pick confirmation without carrier/shipping-label calls, cycle-count variance/tolerance reconciliation, unsafe source/evidence ref refusal, and explicit false robotics/scanner/cloud runtime capability flags.
- **Non-claims:** no durable persistence, no WMS runtime task engine, no procurement three-way match, no accounting ledger mutation, no robotics/scanner runtime I/O, no carrier call, no shipping label generation, no Workflow execution, no statutory filing, no cloud deployment, no production warehouse parity claim, and no runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-warehouse-inventory-domain`; `cargo clippy --locked -p oya-warehouse-inventory-domain --all-targets -- -D warnings`; ERP parity map tests/clippy; enterprise package-group tests/clippy including warehouse/procurement/treasury; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-PRODUCTION-PLANNING-001 — Production planning domain foundation

- **Intent:** Add a flat production-planning domain foundation for approved work definitions, MRP planned-order proposals, and production-release preparation so ERP/SAP PP parity has a repo-native service destination before cloud integration.
- **Primary scope:** `microservices/production-planning/crates/oya-production-planning-domain`, `specs/microservices/production-planning.json`, `registry/catalog/oya-production-planning-domain.yaml`, `Cargo.toml`, Tenant RBAC ERP parity map PP row, evidence/audit closeout.
- **Acceptance:** Domain tests validate work-definition approval with BOM/route/work-center/effective-date evidence, MRP net-requirement and lot-size planned-order derivation, production release material/capacity evidence checks, unsafe source/evidence ref refusal, invalid date/horizon/quantity refusal, and explicit false procurement purchase-order/inventory/shop-floor/accounting/Workflow/cloud runtime capability flags.
- **Non-claims:** no durable persistence, no live MRP engine, no finite scheduler, no manufacturing execution/shop-floor runtime, no inventory mutation, no procurement purchase-order creation, no accounting posting, no Workflow execution, no statutory filing, no cloud deployment, no production PP parity claim, and no runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-production-planning-domain`; `cargo clippy --locked -p oya-production-planning-domain --all-targets -- -D warnings`; ERP parity map tests/clippy; enterprise package-group tests/clippy including production-planning/warehouse/treasury/procurement; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-QUALITY-001 — Quality management domain foundation

- **Intent:** Add a flat quality-management domain foundation for inspection-plan approval, inspection-lot usage decisions, quality-certificate preparation, and quality-notification opening so ERP/SAP QM parity has a repo-native service destination before cloud integration.
- **Primary scope:** `microservices/quality-management/crates/oya-quality-management-domain`, `specs/microservices/quality-management.json`, `registry/catalog/oya-quality-management-domain.yaml`, `Cargo.toml`, Tenant RBAC ERP parity map QM row, evidence/audit closeout.
- **Acceptance:** Domain tests validate inspection plan approval with characteristics, sampling, AQL, effective-date evidence; inspection-lot accepted/rejected usage decisions; quality-certificate preparation from accepted inspection; quality-notification opening from rejected inspection; unsafe source/evidence ref refusal; invalid date/AQL/quantity/result refusal; and explicit false inventory mutation, certificate rendering, email delivery, CAPA Workflow, supplier-network, maintenance-notification, and cloud runtime capability flags.
- **Sources:** SAP S/4HANA Cloud Quality Management / Quality Inspection / quality certificates; Oracle Fusion Cloud Quality inspection plans, inline inspections, and quality issues/actions; Microsoft Dynamics 365 quality/nonconformance, quality orders, and advanced quality management.
- **Non-claims:** no durable persistence, no live inspection runtime, no inventory blocking or release mutation, no laboratory instrument integration, no certificate PDF rendering, no email delivery, no supplier collaboration network, no CAPA Workflow execution, no plant-maintenance notification, no statutory filing, no cloud deployment, no production QM parity claim, and no runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-quality-management-domain`; `cargo clippy --locked -p oya-quality-management-domain --all-targets -- -D warnings`; ERP parity map tests/clippy; enterprise package-group tests/clippy including quality-management/production-planning/warehouse/treasury/procurement; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-PLANT-MAINTENANCE-001 — Plant maintenance domain foundation

- **Intent:** Add a flat plant-maintenance domain foundation for equipment asset registration, preventive-maintenance plan approval, maintenance work-order release, and work-order completion so ERP/SAP PM/EAM parity has a repo-native service destination before cloud integration.
- **Primary scope:** `microservices/plant-maintenance/crates/oya-plant-maintenance-domain`, `specs/microservices/plant-maintenance.json`, `registry/catalog/oya-plant-maintenance-domain.yaml`, `Cargo.toml`, Tenant RBAC ERP parity map PM row, evidence/audit closeout.
- **Acceptance:** Domain tests validate equipment registration with functional location, criticality, installation/warranty evidence; preventive maintenance plan approval with interval/lead-time/labor/spare metadata; work-order release with planned labor/spares/safety permit/job instruction evidence; work-order completion with labor variance, spare-part consumption, downtime, and recalculation flag; unsafe source/evidence ref refusal; invalid date/interval/quantity refusal; missing registration/approval/release refusal; spare-part over-consumption refusal; and explicit false durable-registry, IoT/SCADA, scheduler, inventory, procurement, technician-dispatch, accounting, meter-write, Workflow, cloud, and runtime-audit capability flags.
- **Sources:** SAP S/4HANA Cloud Asset Management configuration/docs, SAP Cloud ERP Asset Management, Oracle Fusion Cloud Maintenance execution/work-order docs, Oracle Maintenance work-order REST docs, and Microsoft Dynamics 365 Asset Management assets/work-orders and maintenance plans docs.
- **Non-claims:** no durable persistence, live EAM runtime, scheduling engine, technician/mobile dispatch, IoT/SCADA ingestion, spare-parts inventory mutation/reservation, procurement requisition creation, accounting posting, plant safety permit execution, Workflow execution, statutory filing, cloud deployment, production PM/EAM parity claim, or runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-plant-maintenance-domain`; `cargo clippy --locked -p oya-plant-maintenance-domain --all-targets -- -D warnings`; ERP parity map tests/clippy; enterprise package-group tests/clippy including plant-maintenance/quality-management/production-planning/warehouse/treasury/procurement; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-ACCOUNTING-004 — Accounting statutory tax rulepack source manifest

- **Intent:** Add a source-versioned accounting statutory rulepack manifest so future ledger, tax workflow, filing, and cloud adapters consume official source provenance without this slice claiming execution.
- **Primary scope:** `microservices/accounting/crates/oya-accounting-journal-domain/src/lib.rs`, `microservices/accounting/crates/oya-accounting-journal-domain/tests/rulepack_manifest.rs`, accounting spec AC-12/D-12, evidence/audit closeout.
- **Acceptance:** Manifest tests validate rulepack ref, accounting period, source version, effective date, approval evidence, official NTS/HomeTax/Law.go.kr/IRS source URLs, source evidence refs, source digests, empty source refusal, unofficial URL refusal, missing version refusal, digest refusal, and ledger/Workflow/filing/payment/cloud overclaim refusal.
- **Non-claims:** no durable ledger persistence, no Workflow execution, no statutory filing rail, no payment execution, no cloud deployment, no production statutory correctness claim, and no runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-accounting-journal-domain`; `cargo clippy --locked -p oya-accounting-journal-domain --all-targets -- -D warnings`; accounting package-group tests/clippy; enterprise package-group tests/clippy; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-HR-008 — HR statutory labor rulepack source manifest

- **Intent:** Add a source-versioned HR statutory rulepack manifest so future HR labor-compliance, Workflow, and durable/cloud adapters consume official source provenance without this slice claiming execution.
- **Primary scope:** `microservices/hr/crates/oya-hr-employment-domain/src/lib.rs`, `microservices/hr/crates/oya-hr-employment-domain/tests/rulepack_manifest.rs`, HR spec AC-14/D-16, evidence/audit closeout.
- **Acceptance:** Manifest tests validate rulepack ref, source version, effective date, approval evidence, official MOEL/Law.go.kr/DOL/EEOC source URLs, source evidence refs, source digests, empty source refusal, unofficial URL refusal, missing version refusal, digest refusal, and Workflow/payroll/filing/cloud overclaim refusal.
- **Non-claims:** no Workflow engine execution, no payroll calculation, no statutory filing rail, no storage adapter change, no cloud deployment, no production statutory correctness claim, and no runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-hr-employment-domain`; `cargo clippy --locked -p oya-hr-employment-domain --all-targets -- -D warnings`; HR package-group tests/clippy; enterprise package-group tests/clippy; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-PAYROLL-005 — Payroll statutory rulepack source manifest

- **Intent:** Add a source-versioned statutory rulepack manifest to the payroll domain so future tax calculation and filing adapters consume official, evidence-backed annual sources without this slice claiming calculations or filing execution.
- **Primary scope:** `microservices/payroll/crates/oya-payroll-run-domain/src/lib.rs`, `microservices/payroll/crates/oya-payroll-run-domain/tests/rulepack_manifest.rs`, payroll spec AC-13/D-12, evidence/audit closeout.
- **Acceptance:** Manifest tests validate rulepack ref, payroll period, source version, effective date, official source URLs, source evidence refs, source digests, US federal and Korea source examples, empty source refusal, unofficial URL refusal, and capability-overclaim refusal.
- **Non-claims:** no tax calculation engine, no filing rail, no disbursement rail, no cloud deployment, no production statutory correctness claim, and no runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-payroll-run-domain`; `cargo clippy --locked -p oya-payroll-run-domain --all-targets -- -D warnings`; payroll package-group tests/clippy; enterprise package-group tests/clippy; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-ACCOUNTING-001 — Accounting journal foundation

Disjoint path envelope:

- `microservices/accounting/README.md`
- `microservices/accounting/PRD.md`
- `microservices/accounting/crates/oya-accounting-journal-domain/**`
- `microservices/accounting/catalog/oya-accounting-journal-domain.yaml`
- `registry/catalog/oya-accounting-journal-domain.yaml`
- `specs/microservices/accounting.json`
- `Cargo.toml`

Acceptance focus: chart-of-accounts identity, balanced journal voucher posting, open-period guard, payroll source digest requirement, KR VAT workflow evidence, AP approval gates, and immutable close evidence refusal.

Verification:

- `cargo test -p oya-accounting-journal-domain`
- `cargo clippy -p oya-accounting-journal-domain --all-targets -- -D warnings`
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-accounting-journal-domain-foundation-1779522601.json --severity error`


### CS-ENT-ACCOUNTING-002 — Accounting HTTP runtime adapter foundation

- **Intent:** Bind existing Accounting API DTO/OpenAPI preview routes to the repo-native Hyper router/middleware foundation without adding a deployed listener, ledger storage, Workflow execution, statutory filing rails, payment execution, Payroll call, or runtime audit emitter.
- **Primary scope:** `microservices/accounting/crates/oya-accounting-journal-runtime`, `Cargo.toml`, `registry/catalog/oya-accounting-journal-runtime.yaml`, Accounting spec AC-10/D-10, Accounting OpenAPI metadata, evidence/audit closeout.
- **Acceptance:** Runtime tests dispatch journal posting, payroll posting, VAT workflow planning, invalid JSON/domain errors, route manifest, bounded server config, and health non-claims through transport-neutral `HttpRequest`/`HttpResponse`.
- **Non-claims:** no live listener deployment, ledger storage, Workflow execution, statutory filing rails, payment execution, Payroll network calls, runtime audit-chain emission, or cloud runtime.
- **Verification:** `cargo test --locked -p oya-accounting-journal-runtime`; `cargo clippy --locked -p oya-accounting-journal-runtime --all-targets -- -D warnings`; `cargo fmt --all -- --check`; Accounting OpenAPI semver; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.


### CS-ENT-ACCOUNTING-003 — Accounting in-memory storage seam reference

- **Intent:** Add a volatile in-memory accounting journal storage port/adapter reference for journal-post audit, payroll-posting audit, and VAT Workflow dispatch metadata without durable ledger storage or runtime side effects.
- **Primary scope:** `microservices/accounting/crates/oya-accounting-journal-storage-adapter-inmemory`, `Cargo.toml`, `registry/catalog/oya-accounting-journal-storage-adapter-inmemory.yaml`, accounting spec AC-11/D-11, evidence/audit closeout.
- **Acceptance:** Storage tests persist all accounting metadata record kinds, preserve topic/tenant/legal-entity/primary refs, evidence counts, payload data-class labels, and idempotency keys, reject duplicate or unsafe idempotency keys, and expose capability flags that keep durable ledger backend, Postgres/RLS, Workflow execution, statutory filing rails, payment execution, Payroll calls, and audit-chain emission false.
- **Non-claims:** volatile test/local reference only; no durable ledger backend, Postgres/RLS, Workflow execution, statutory filing transport, payment execution, Payroll network call, cloud integration, or runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-accounting-journal-storage-adapter-inmemory`; `cargo clippy --locked -p oya-accounting-journal-storage-adapter-inmemory --all-targets -- -D warnings`; accounting package-group tests/clippy; enterprise package-group tests/clippy; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-HR-002 — HR leave approval payroll-impact metadata foundation

Disjoint path envelope:

- `microservices/hr/crates/oya-hr-employment-domain/src/lib.rs`
- `microservices/hr/crates/oya-hr-employment-domain/tests/leave.rs`
- `microservices/hr/crates/oya-hr-employment-app/src/lib.rs`
- `microservices/hr/crates/oya-hr-employment-app/tests/leave.rs`
- `specs/microservices/hr.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-hr-leave-payroll-impact-1779532800.json`
- `evidence/vcs/cs-ent-hr-leave-payroll-impact-oya-vcs-lifecycle-20260523.json`

Acceptance focus: HR AC-04 pure-domain leave decision payroll-impact plan and
metadata-only app envelope. The domain requires tenant/legal-entity/employee/
approver identifiers, manager delegation/escalation routing evidence, Workflow
ref, labor-law rulepack basis, decision evidence, payroll period, payroll
impact kind, and payroll-impact audit evidence. The app crate emits a
metadata-only HR-to-payroll integration envelope. This slice makes no leave
balance calculation, payroll calculation, storage, Workflow execution, audit
chain emission, HTTP adapter, or cloud runtime claim.

Verification:

- `cargo test --locked -p oya-hr-employment-domain -p oya-hr-employment-app`
- `cargo clippy --locked -p oya-hr-employment-domain -p oya-hr-employment-app --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- enterprise package-group regression tests/clippy across HR, Payroll, Accounting, and Tenant RBAC crates
- data-class annotation scan for touched HR domain/app public fields
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-hr-leave-payroll-impact-1779532800.json --severity error`


### CS-ENT-HR-003 — HR sensitive-read purpose-bound policy foundation

Disjoint path envelope:

- `microservices/hr/crates/oya-hr-employment-domain/src/lib.rs`
- `microservices/hr/crates/oya-hr-employment-domain/tests/privacy.rs`
- `microservices/hr/crates/oya-hr-employment-app/src/lib.rs`
- `microservices/hr/crates/oya-hr-employment-app/tests/privacy.rs`
- `specs/microservices/hr.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-hr-sensitive-read-policy-1779533400.json`
- `evidence/vcs/cs-ent-hr-sensitive-read-policy-oya-vcs-lifecycle-20260523.json`

Acceptance focus: HR AC-05 pure-domain sensitive HR read policy evaluation and
metadata-only app audit envelope. The domain requires purpose-bound reads, a
non-empty legal basis, policy ref, basis/request/read-log audit evidence, and
consent evidence whenever consent is the legal basis. The app crate emits a
metadata-only sensitive-read audit envelope for future authorization/audit
adapters. This slice makes no runtime authorization middleware, data retrieval,
storage, audit-chain emission, HTTP adapter, or cloud runtime claim.

Verification:

- `cargo test --locked -p oya-hr-employment-domain -p oya-hr-employment-app`
- `cargo clippy --locked -p oya-hr-employment-domain -p oya-hr-employment-app --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- enterprise package-group regression tests/clippy across HR, Payroll, Accounting, and Tenant RBAC crates
- data-class annotation scan for touched HR domain/app public fields
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-hr-sensitive-read-policy-1779533400.json --severity error`


### CS-ENT-HR-004 — HR sensitive-read API DTO and OpenAPI preview contract

Disjoint path envelope:

- `microservices/hr/crates/oya-hr-employment-api/src/lib.rs`
- `microservices/hr/crates/oya-hr-employment-api/tests/contracts.rs`
- `microservices/hr/contracts/openapi-v1.yaml`
- `microservices/hr/contracts/openapi-v1.meta.yaml`
- `specs/microservices/hr.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-hr-sensitive-read-api-openapi-1779534000.json`
- `evidence/vcs/cs-ent-hr-sensitive-read-api-openapi-oya-vcs-lifecycle-20260523.json`

Acceptance focus: HR AC-10 transport-neutral DTO and OpenAPI preview contract
for the AC-05 sensitive-read policy foundation. DTOs preserve camelCase
request fields, stable SCREAMING_SNAKE_CASE enum labels, deterministic
conversion into `SensitiveHrReadInput`, and a metadata-only decision response.
The OpenAPI 3.2.0 preview adds `/hr/v1/sensitive-read-policy-decisions` for
later cloud adapters. This slice makes no deployed HTTP endpoint, runtime
authorization middleware, sensitive HR data retrieval, storage, audit-chain
emission, or cloud runtime claim.

Verification:

- `cargo test --locked -p oya-hr-employment-api`
- `cargo clippy --locked -p oya-hr-employment-api --all-targets -- -D warnings`
- `./bin/oya gate validate api-semver --contracts-dir microservices/hr/contracts`
- `cargo fmt --all -- --check`
- enterprise package-group regression tests/clippy across HR, Payroll, Accounting, and Tenant RBAC crates
- data-class annotation scan for touched HR API public fields and OpenAPI `x-data-class` sensitive-read fields
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-hr-sensitive-read-api-openapi-1779534000.json --severity error`


### CS-ENT-HR-005 — HR leave payroll-impact API DTO and OpenAPI preview contract

Disjoint path envelope:

- `microservices/hr/crates/oya-hr-employment-api/src/lib.rs`
- `microservices/hr/crates/oya-hr-employment-api/tests/contracts.rs`
- `microservices/hr/contracts/openapi-v1.yaml`
- `microservices/hr/contracts/openapi-v1.meta.yaml`
- `specs/microservices/hr.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-hr-leave-payroll-impact-api-openapi-1779534600.json`
- `evidence/vcs/cs-ent-hr-leave-payroll-impact-api-openapi-oya-vcs-lifecycle-20260523.json`

Acceptance focus: HR AC-11 transport-neutral DTO and OpenAPI preview contract
for the AC-04 leave/payroll-impact foundation. DTOs preserve camelCase request
fields, stable SCREAMING_SNAKE_CASE leave/payroll enum labels, deterministic
conversion into `LeavePayrollImpactInput`, and a metadata-only HR-to-payroll
response. The OpenAPI 3.2.0 preview adds
`/hr/v1/leave-payroll-impact-plans` for later cloud/payroll adapters. This
slice makes no deployed HTTP endpoint, leave balance calculation, payroll
calculation, storage, Workflow execution, audit-chain emission, or cloud runtime
claim.

Verification:

- `cargo test --locked -p oya-hr-employment-api`
- `cargo clippy --locked -p oya-hr-employment-api --all-targets -- -D warnings`
- `./bin/oya gate validate api-semver --contracts-dir microservices/hr/contracts`
- `cargo fmt --all -- --check`
- enterprise package-group regression tests/clippy across HR, Payroll, Accounting, and Tenant RBAC crates
- data-class annotation scan for touched HR API public fields and OpenAPI `x-data-class` leave/payroll fields
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-hr-leave-payroll-impact-api-openapi-1779534600.json --severity error`


### CS-ENT-HR-006 — HR HTTP runtime adapter foundation

- **Intent:** Bind existing HR API DTO/OpenAPI preview routes to the repo-native Hyper router/middleware foundation without adding a deployed listener, persistence, Workflow execution, Payroll call, or sensitive-data retrieval.
- **Primary scope:** `microservices/hr/crates/oya-hr-employment-runtime`, `Cargo.toml`, `registry/catalog/oya-hr-employment-runtime.yaml`, HR spec AC-12/D-14, HR OpenAPI metadata, evidence/audit closeout.
- **Acceptance:** Runtime tests dispatch onboarding, labor-compliance workflow planning, sensitive-read policy decision, leave payroll-impact planning, invalid JSON, forbidden sensitive purpose, route manifest, bounded server config, and health non-claims through transport-neutral `HttpRequest`/`HttpResponse`.
- **Non-claims:** no live listener deployment, storage, Workflow execution, Payroll network calls, sensitive HR data fetch, runtime audit-chain emission, or cloud runtime.
- **Verification:** `cargo test --locked -p oya-hr-employment-runtime`; `cargo clippy --locked -p oya-hr-employment-runtime --all-targets -- -D warnings`; `cargo fmt --all -- --check`; HR OpenAPI semver; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.



### CS-ENT-SUITE-010 — Enterprise local runtime composition manifest

- **Intent:** Catalog router-ready HR, Payroll, Accounting, and Tenant RBAC runtime route manifests in one local composition crate for later cloud listener wiring without deploying a listener or executing runtime side effects.
- **Primary scope:** `microservices/tenant-rbac/crates/oya-tenant-rbac-local-runtime-composition`, `Cargo.toml`, `registry/catalog/oya-tenant-rbac-local-runtime-composition.yaml`, Tenant RBAC spec AC-12/D-09, evidence/audit closeout.
- **Acceptance:** Composition tests preserve service, method, path, operation id, request/response data-class metadata, route count, and method/path uniqueness while capability flags keep deployed listener, authentication runtime, child network calls, storage integration, Workflow execution, cloud deployment, and runtime audit-chain emission false.
- **Non-claims:** no deployed HTTP listener, authentication runtime, child-service network calls, storage integration, Workflow engine execution, cloud deployment, or runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-tenant-rbac-local-runtime-composition`; `cargo clippy --locked -p oya-tenant-rbac-local-runtime-composition --all-targets -- -D warnings`; Tenant RBAC package-group tests/clippy; enterprise package-group tests/clippy; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.


### CS-ENT-SUITE-011 — Enterprise local in-memory service harness

- **Intent:** Compose the HR, Payroll, Accounting, and Tenant RBAC in-memory adapters into one process-local harness that records real app-layer envelopes for later cloud-integration rehearsal without deployed runtime side effects.
- **Primary scope:** `microservices/tenant-rbac/crates/oya-tenant-rbac-local-inmemory-harness`, `Cargo.toml`, `registry/catalog/oya-tenant-rbac-local-inmemory-harness.yaml`, Tenant RBAC spec AC-13/D-10, evidence/audit closeout.
- **Acceptance:** Harness tests persist HR leave payroll-impact, Payroll HR leave intake, Payroll accounting dispatch, Accounting payroll posting, and Tenant RBAC Workflow dispatch metadata into service-specific in-memory stores/queue, expose aggregate record counts and capability flags, surface duplicate-store errors, and keep durable storage, Postgres/RLS, deployed listener, child network calls, Workflow engine, broker publish, filing/disbursement rails, cloud deployment, and runtime audit-chain emission false.
- **Non-claims:** no durable backend, Postgres/RLS, deployed listener, child-service network calls, Workflow engine/broker execution, statutory filing/disbursement rails, cloud deployment, runtime storage write path, or runtime audit-chain emission.
- **Verification:** `cargo test --locked -p oya-tenant-rbac-local-inmemory-harness`; `cargo clippy --locked -p oya-tenant-rbac-local-inmemory-harness --all-targets -- -D warnings`; Tenant RBAC package-group tests/clippy; enterprise package-group tests/clippy; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.


### CS-ENT-SUITE-012 — Executable ERP/SAP parity composition map

- **Intent:** Convert ADR-0315 SAP module parity into a typed, testable Tenant RBAC composition map that preserves flat service ownership and forbids a monolithic ERP service while preparing later Oyatie cloud integration sequencing.
- **Primary scope:** `microservices/tenant-rbac/crates/oya-tenant-rbac-erp-parity-map`, `Cargo.toml`, `registry/catalog/oya-tenant-rbac-erp-parity-map.yaml`, Tenant RBAC spec AC-14/D-11, evidence/audit closeout.
- **Acceptance:** Parity tests cover all 23 SAP module rows, first-write owners, service destinations, HCM/FI evidence refs to landed HR/payroll/accounting/harness slices, new-required flat ERP operational gaps, capability non-claims, and rejection of `microservices/erp` destinations.
- **Non-claims:** no monolithic ERP microservice, deployed listener, durable business-document store, Workflow execution, cloud deployment, runtime audit-chain emission, or production ERP parity claim.
- **Verification:** `cargo test --locked -p oya-tenant-rbac-erp-parity-map`; `cargo clippy --locked -p oya-tenant-rbac-erp-parity-map --all-targets -- -D warnings`; Tenant RBAC package-group tests/clippy; enterprise package-group tests/clippy; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.


### CS-ENT-SUITE-013 — Enterprise cloud readiness gate

- **Intent:** Combine the local runtime composition, in-memory service harness, and executable ERP parity map into a deterministic pre-cloud gate that proves local rehearsal readiness while refusing cloud deployment overclaims.
- **Primary scope:** `microservices/tenant-rbac/crates/oya-tenant-rbac-cloud-readiness-gate`, `Cargo.toml`, `registry/catalog/oya-tenant-rbac-cloud-readiness-gate.yaml`, Tenant RBAC spec AC-15/D-12, evidence/audit closeout.
- **Acceptance:** Readiness tests assert route count, SAP module count, local rehearsal gates, evidence refs, blocker inventory, and negative cloud-claim validation when blockers remain unresolved.
- **Non-claims:** no deployed listener, auth runtime, durable business store, Postgres/RLS, Workflow engine, broker publish, statutory filing/disbursement rails, runtime audit emission, cloud deployment, or SLO evidence.
- **Verification:** `cargo test --locked -p oya-tenant-rbac-cloud-readiness-gate`; `cargo clippy --locked -p oya-tenant-rbac-cloud-readiness-gate --all-targets -- -D warnings`; Tenant RBAC package-group tests/clippy; enterprise package-group tests/clippy; `cargo fmt --all -- --check`; dependency-seam evidence gate; append-only audit-chain check; Oya VCS claim/status/verify/done/promote transcript.

### CS-ENT-HR-007 — HR in-memory storage seam reference

Disjoint path envelope:

- `microservices/hr/crates/oya-hr-employment-storage-adapter-inmemory/**`
- `registry/catalog/oya-hr-employment-storage-adapter-inmemory.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `specs/microservices/hr.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-hr-storage-adapter-inmemory-1779539400.json`
- `evidence/vcs/cs-ent-hr-storage-adapter-inmemory-oya-vcs-lifecycle-20260523.json`

Acceptance focus: HR AC-13 storage seam reference for employment lifecycle,
labor workflow, leave payroll-impact, and sensitive-read policy metadata. The
adapter records app-layer envelopes with topic, tenant/entity, primary refs,
evidence counts, payload data-class labels, idempotency validation, and duplicate
write refusal. It is an in-memory reference only: no durable backend,
Postgres/RLS, sensitive data retrieval, Workflow execution, Payroll network call,
audit-chain runtime emission, or cloud storage claim is made.

Verification:

- `cargo test --locked -p oya-hr-employment-storage-adapter-inmemory`
- `cargo clippy --locked -p oya-hr-employment-storage-adapter-inmemory --all-targets -- -D warnings`
- `cargo test --locked -p oya-hr-employment-domain -p oya-hr-employment-app -p oya-hr-employment-api -p oya-hr-employment-runtime -p oya-hr-employment-storage-adapter-inmemory`
- `cargo clippy --locked -p oya-hr-employment-domain -p oya-hr-employment-app -p oya-hr-employment-api -p oya-hr-employment-runtime -p oya-hr-employment-storage-adapter-inmemory --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- data-class annotation scan for HR storage public fields
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-hr-storage-adapter-inmemory-1779539400.json --severity error`

### CS-ENT-APP-001 — HR/payroll/accounting app-layer integration envelopes

Disjoint path envelope:

- `microservices/hr/crates/oya-hr-employment-app/**`
- `microservices/payroll/crates/oya-payroll-run-app/**`
- `microservices/accounting/crates/oya-accounting-journal-app/**`
- `microservices/{hr,payroll,accounting}/catalog/*-app.yaml`
- `registry/catalog/oya-hr-employment-app.yaml`
- `registry/catalog/oya-payroll-run-app.yaml`
- `registry/catalog/oya-accounting-journal-app.yaml`
- `specs/microservices/{hr,payroll,accounting}.json`
- `Cargo.toml`

Acceptance focus: app crates orchestrate only the pure domain crates and the
data-boundary kernel, returning metadata-only audit, Workflow dispatch, and
cross-service integration envelopes. They intentionally do not perform storage,
network, workflow dispatch, audit-chain emission, statutory filing, or cloud
runtime I/O.

Verification:

- `cargo test -p oya-hr-employment-app -p oya-payroll-run-app -p oya-accounting-journal-app`
- `cargo clippy -p oya-hr-employment-app -p oya-payroll-run-app -p oya-accounting-journal-app --all-targets -- -D warnings`
- `cargo tree -p oya-hr-employment-app`, `cargo tree -p oya-payroll-run-app`, and `cargo tree -p oya-accounting-journal-app`
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-app-integration-envelopes-1779527000.json --severity error`

### CS-ENT-API-001 — HR/payroll/accounting API DTO contracts

Disjoint path envelope:

- `microservices/hr/crates/oya-hr-employment-api/**`
- `microservices/payroll/crates/oya-payroll-run-api/**`
- `microservices/accounting/crates/oya-accounting-journal-api/**`
- `microservices/{hr,payroll,accounting}/catalog/*-api.yaml`
- `registry/catalog/oya-hr-employment-api.yaml`
- `registry/catalog/oya-payroll-run-api.yaml`
- `registry/catalog/oya-accounting-journal-api.yaml`
- `registry/dependency-rationales.json`
- `specs/microservices/{hr,payroll,accounting}.json`
- `Cargo.toml`

Acceptance focus: serializable, camelCase request/error DTOs with stable
SCREAMING_SNAKE_CASE enum values convert deterministically into existing
domain/app inputs. These API crates are contract/admission surfaces only: no
HTTP framework, router, auth middleware, storage, Workflow client, audit emitter,
or cloud runtime is introduced.

Verification:

- `cargo test --locked -p oya-hr-employment-api -p oya-payroll-run-api -p oya-accounting-journal-api`
- `cargo clippy --locked -p oya-hr-employment-api --all-targets -- -D warnings` plus payroll/accounting API equivalents
- `cargo tree -p oya-hr-employment-api`, `cargo tree -p oya-payroll-run-api`, and `cargo tree -p oya-accounting-journal-api`
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-api-dto-contracts-1779527400.json --severity error`

### CS-ENT-OPENAPI-001 — HR/payroll/accounting OpenAPI contract surfaces

Disjoint path envelope:

- `microservices/hr/contracts/openapi-v1.yaml`
- `microservices/hr/contracts/openapi-v1.meta.yaml`
- `microservices/payroll/contracts/openapi-v1.yaml`
- `microservices/payroll/contracts/openapi-v1.meta.yaml`
- `microservices/accounting/contracts/openapi-v1.yaml`
- `microservices/accounting/contracts/openapi-v1.meta.yaml`
- `specs/microservices/{hr,payroll,accounting}.json`
- `tasks/enterprise-microservices-*`

Acceptance focus: OpenAPI 3.2.0 wire-shape contracts align to the Rust API
DTO crates and carry semver metadata. The contracts are preview, not deployed:
no HTTP server, auth enforcement, storage, Workflow execution, filing transport,
or cloud adapter is claimed.

Verification:

- JSON/YAML structural parse for all three OpenAPI artifacts.
- `./bin/oya gate validate api-semver --contracts-dir microservices/hr/contracts` plus payroll/accounting equivalents.
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-openapi-contracts-1779528000.json --severity error`


### CS-ENT-CICD-001 — HR/payroll/accounting Jenkins quality gates

Disjoint path envelope:

- `microservices/hr/ci/Jenkinsfile`
- `microservices/payroll/ci/Jenkinsfile`
- `microservices/accounting/ci/Jenkinsfile`
- `specs/microservices/{hr,payroll,accounting}.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-cicd-quality-gates-1779528600.json`
- `evidence/vcs/cs-ent-cicd-quality-gates-oya-vcs-lifecycle-20260523.json`

Acceptance focus: Jenkins LTS self-hostable CI parity per ADR-0349 for the
three enterprise services. Each Jenkinsfile runs checkout, Rust CI tooling,
workspace fmt, per-service package-group check/clippy/nextest, OpenAPI semver,
Oya VCS admission, and Wave 15-ZE CI evidence archival. This slice makes no
runtime deploy, ArgoCD sync, Helm chart, storage, Workflow execution, statutory
filing, or cloud adapter claim.

Verification:

- Static Jenkins lane scan over `microservices/{hr,payroll,accounting}/ci/Jenkinsfile`.
- `cargo fmt --all -- --check`
- `cargo test --locked -p oya-hr-employment-domain -p oya-hr-employment-app -p oya-hr-employment-api -p oya-payroll-run-domain -p oya-payroll-run-app -p oya-payroll-run-api -p oya-accounting-journal-domain -p oya-accounting-journal-app -p oya-accounting-journal-api`
- `cargo clippy --locked` for the same package set with `--all-targets -- -D warnings`.
- `./bin/oya gate validate api-semver --contracts-dir microservices/{hr,payroll,accounting}/contracts`
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-cicd-quality-gates-1779528600.json --severity error`


### CS-ENT-SUITE-001 — Tenant RBAC governance foundation

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-domain/**`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-usecase/**`
- `microservices/tenant-rbac/catalog/oya-tenant-rbac-{domain,app}.yaml`
- `registry/catalog/oya-tenant-rbac-{domain,app}.yaml`
- `specs/microservices/tenant-rbac.json`
- `Cargo.toml`
- `Cargo.lock`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-foundation-1779529200.json`
- `evidence/vcs/cs-ent-platform-foundation-oya-vcs-lifecycle-20260523.json`

Acceptance focus: platform-level policy admission and group close projection for
HR, Payroll, and Accounting children. The domain crate refuses child writes
that bypass the shared tenant RBAC policy gateway or omit data-class/audit evidence,
and preserves legal-entity close boundaries during group rollup. The app crate
emits metadata-only ops command envelopes and refuses manual SSH routing.
This slice makes no storage, network, REST, Workflow execution, incident
runtime, statutory filing, ArgoCD, Helm, or cloud adapter claim.

Verification:

- `cargo test --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase`
- `cargo clippy --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `./bin/oya gate validate cargo-prefix --workspace Cargo.toml --prefix oya-`
- `./bin/oya gate validate slo-coverage --registry registry/catalog`
- `./bin/oya gate validate claim-ceiling --registry registry/catalog`
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-foundation-1779529200.json --severity error`


### CS-ENT-SUITE-002 — Tenant RBAC cross-product Workflow deterministic gates

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-domain/src/lib.rs`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-domain/tests/workflow.rs`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-usecase/src/lib.rs`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-usecase/tests/workflow.rs`
- `specs/microservices/tenant-rbac.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-tenant-rbac-workflow-gates-1779529800.json`
- `evidence/vcs/cs-ent-tenant-rbac-workflow-gates-oya-vcs-lifecycle-20260523.json`

Acceptance focus: AC-04 foundation for workflows that cross HR, Payroll, and
Accounting. The domain crate requires Workflow-owned routing, Object
Graph-owned typed relationships, all deterministic gate evidence, and refuses
AI suggestion close authority. The app crate emits a metadata-only Workflow
dispatch envelope for later adapters. This slice makes no Workflow execution,
Object Graph persistence, child service call, storage, REST, incident runtime,
ArgoCD, Helm, or cloud adapter claim.

Verification:

- `cargo test --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase`
- `cargo clippy --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-tenant-rbac-workflow-gates-1779529800.json --severity error`


### CS-ENT-SUITE-003 — Tenant RBAC incident rollback/quarantine envelope

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-domain/src/lib.rs`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-domain/tests/incident.rs`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-usecase/src/lib.rs`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-usecase/tests/incident.rs`
- `specs/microservices/tenant-rbac.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-incident-rollback-1779530400.json`
- `evidence/vcs/cs-ent-platform-incident-rollback-oya-vcs-lifecycle-20260523.json`

Acceptance focus: AC-05 metadata foundation for unhealthy tenant RBAC runtime
incidents. The domain crate requires rollback or quarantine before remediation,
requires canary/incident/rollback audit evidence, refuses manual SSH, and
requires infra/config convergence through OpenTofu or ops references. The app
crate emits a metadata-only incident rollback envelope for later adapters. This
slice makes no runtime rollback execution, incident emitter, storage, REST,
ArgoCD, Helm, Workflow execution, or cloud adapter claim.

Verification:

- `cargo test --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase`
- `cargo clippy --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- package-group regression tests/clippy across HR, Payroll, Accounting, and Tenant RBAC crates
- data-class annotation scan for Tenant RBAC public fields
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-incident-rollback-1779530400.json --severity error`


### CS-ENT-SUITE-004 — Tenant RBAC API DTO contracts

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-api/**`
- `microservices/tenant-rbac/catalog/oya-tenant-rbac-api.yaml`
- `registry/catalog/oya-tenant-rbac-api.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `registry/dependency-rationales.json`
- `specs/microservices/tenant-rbac.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-api-dto-contracts-1779531000.json`
- `evidence/vcs/cs-ent-platform-api-dto-contracts-oya-vcs-lifecycle-20260523.json`

Acceptance focus: AC-06 preview DTO contracts for Tenant RBAC policy
admission, group rollup, cross-product Workflow planning, incident rollback,
and ops command metadata. DTOs serialize camelCase JSON with stable
SCREAMING_SNAKE_CASE enums and convert into existing domain/app inputs. This
slice makes no HTTP server, auth middleware, storage, Workflow execution,
audit-chain emission, incident runtime, OpenTofu execution, or cloud adapter
claim.

Verification:

- `cargo test --locked -p oya-tenant-rbac-api`
- `cargo clippy --locked -p oya-tenant-rbac-api --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- package-group regression tests/clippy across HR, Payroll, Accounting, and Tenant RBAC crates
- data-class annotation scan for Tenant RBAC API public fields
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-api-dto-contracts-1779531000.json --severity error`


### CS-ENT-SUITE-005 — Tenant RBAC OpenAPI preview contracts

Disjoint path envelope:

- `microservices/tenant-rbac/contracts/openapi-v1.yaml`
- `microservices/tenant-rbac/contracts/openapi-v1.meta.yaml`
- `specs/microservices/tenant-rbac.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-openapi-contracts-1779531600.json`
- `evidence/vcs/cs-ent-platform-openapi-contracts-oya-vcs-lifecycle-20260523.json`

Acceptance focus: AC-07 OpenAPI 3.2.0 preview wire-shape contract aligned
to `oya-tenant-rbac-api` DTOs for policy admission, group close rollup,
cross-product Workflow planning, incident rollback planning, and ops command
metadata. This slice makes no deployed HTTP endpoint, auth enforcement runtime,
storage adapter, Workflow execution, incident rollback runtime, OpenTofu
execution, or cloud adapter claim.

Verification:

- JSON/YAML structural parse for Tenant RBAC OpenAPI artifacts.
- `./bin/oya gate validate api-semver --contracts-dir microservices/tenant-rbac/contracts`
- `cargo test --locked -p oya-tenant-rbac-api`
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-openapi-contracts-1779531600.json --severity error`


### CS-ENT-SUITE-006 — Tenant RBAC Jenkins quality gate

Disjoint path envelope:

- `microservices/tenant-rbac/ci/Jenkinsfile`
- `specs/microservices/tenant-rbac.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-cicd-quality-gate-1779532200.json`
- `evidence/vcs/cs-ent-platform-cicd-quality-gate-oya-vcs-lifecycle-20260523.json`

Acceptance focus: AC-08 Jenkins LTS self-hostable quality gate per ADR-0349
for Tenant RBAC. The Jenkinsfile runs checkout, Rust CI tooling, workspace
fmt, package-group check/clippy/nextest, OpenAPI semver, Oya VCS admission, and
Wave 15-ZE evidence archival. This slice makes no live Jenkins controller run,
ArgoCD Application, Helm chart, runtime deployment, storage, Workflow execution,
incident runtime, OpenTofu execution, or cloud adapter claim.

Verification:

- Static Jenkins lane scan over `microservices/tenant-rbac/ci/Jenkinsfile`.
- `cargo fmt --all -- --check`
- `cargo test --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase -p oya-tenant-rbac-api`
- `cargo clippy --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase -p oya-tenant-rbac-api --all-targets -- -D warnings`
- `./bin/oya gate validate api-semver --contracts-dir microservices/tenant-rbac/contracts`
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-cicd-quality-gate-1779532200.json --severity error`


### CS-ENT-SUITE-007 — Tenant RBAC HTTP runtime adapter foundation

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-app/**`
- `registry/catalog/oya-tenant-rbac-app.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `registry/dependency-rationales.json`
- `specs/microservices/tenant-rbac.json`
- `microservices/tenant-rbac/contracts/openapi-v1.meta.yaml`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-runtime-adapter-foundation-1779537600.json`
- `evidence/vcs/cs-ent-platform-runtime-adapter-foundation-oya-vcs-lifecycle-20260523.json`

Acceptance focus: AC-09 repo-native Hyper runtime adapter foundation for
Tenant RBAC. The runtime dispatches policy admission, group close rollup,
cross-product Workflow planning, incident rollback planning, ops command
metadata, and health checks to existing API/domain/app seams with OpenAPI-aligned
JSON responses and validation error envelopes. This slice makes no live listener
deployment, auth enforcement runtime, storage, Workflow execution, OpenTofu
execution, incident rollback execution, child-service network calls, cloud
integration, or runtime audit-chain emission claim.

Verification:

- `cargo test --locked -p oya-tenant-rbac-app`
- `cargo clippy --locked -p oya-tenant-rbac-app --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase -p oya-tenant-rbac-api -p oya-tenant-rbac-app`
- `cargo clippy --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase -p oya-tenant-rbac-api -p oya-tenant-rbac-app --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- data-class annotation scan for Tenant RBAC runtime public fields
- `./bin/oya gate validate api-semver --contracts-dir microservices/tenant-rbac/contracts`
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-runtime-adapter-foundation-1779537600.json --severity error`


### CS-ENT-SUITE-008 — Tenant RBAC in-memory storage seam reference

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-storage-adapter-inmemory/**`
- `registry/catalog/oya-tenant-rbac-storage-adapter-inmemory.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `specs/microservices/tenant-rbac.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-tenant-rbac-storage-adapter-inmemory-1779538200.json`
- `evidence/vcs/cs-ent-tenant-rbac-storage-adapter-inmemory-oya-vcs-lifecycle-20260523.json`

Acceptance focus: AC-10 storage seam reference for Tenant RBAC metadata.
The adapter records policy admission, group close rollup, cross-product Workflow
plan, incident rollback plan, and ops command metadata with validated
idempotency keys and duplicate-write refusal. It is an in-memory reference only:
no durable backend, Postgres/RLS, cloud object store, runtime write path,
Workflow execution, OpenTofu execution, incident rollback execution, child-service
network calls, audit-chain runtime emission, or cloud storage claim is made.

Verification:

- `cargo test --locked -p oya-tenant-rbac-storage-adapter-inmemory`
- `cargo clippy --locked -p oya-tenant-rbac-storage-adapter-inmemory --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase -p oya-tenant-rbac-api -p oya-tenant-rbac-app -p oya-tenant-rbac-storage-adapter-inmemory`
- `cargo clippy --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase -p oya-tenant-rbac-api -p oya-tenant-rbac-app -p oya-tenant-rbac-storage-adapter-inmemory --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- data-class annotation scan for Tenant RBAC storage public fields
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-tenant-rbac-storage-adapter-inmemory-1779538200.json --severity error`


### CS-ENT-SUITE-009 — Tenant RBAC in-memory Workflow dispatch queue reference

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-workflow-adapter-inmemory/**`
- `registry/catalog/oya-tenant-rbac-workflow-adapter-inmemory.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `specs/microservices/tenant-rbac.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-tenant-rbac-workflow-adapter-inmemory-1779538800.json`
- `evidence/vcs/cs-ent-tenant-rbac-workflow-adapter-inmemory-oya-vcs-lifecycle-20260523.json`

Acceptance focus: AC-11 Workflow dispatch seam reference for Tenant RBAC
cross-product workflow metadata. The adapter records prepared Workflow dispatch
intents with required gate counts, evidence counts, object-graph relationship
refs, AI suggestion refs, idempotency validation, and duplicate-dispatch
refusal. It is an in-memory reference only: no durable queue, Workflow engine
execution, broker publish, runtime execution, child-service network calls,
audit-chain runtime emission, or cloud Workflow claim is made.

Verification:

- `cargo test --locked -p oya-tenant-rbac-workflow-adapter-inmemory`
- `cargo clippy --locked -p oya-tenant-rbac-workflow-adapter-inmemory --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase -p oya-tenant-rbac-api -p oya-tenant-rbac-app -p oya-tenant-rbac-storage-adapter-inmemory -p oya-tenant-rbac-workflow-adapter-inmemory`
- `cargo clippy --locked -p oya-tenant-rbac-domain -p oya-tenant-rbac-usecase -p oya-tenant-rbac-api -p oya-tenant-rbac-app -p oya-tenant-rbac-storage-adapter-inmemory -p oya-tenant-rbac-workflow-adapter-inmemory --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- data-class annotation scan for Tenant RBAC Workflow adapter public fields
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-tenant-rbac-workflow-adapter-inmemory-1779538800.json --severity error`

### CS-ENT-CRM-001 — CRM customer engagement domain foundation

Disjoint path envelope:

- `microservices/crm/crates/oya-crm-customer-engagement-domain/**`
- `registry/catalog/oya-crm-customer-engagement-domain.yaml`
- `specs/microservices/crm.json`
- `Cargo.toml`
- `Cargo.lock`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-erp-parity-map/**`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-crm-customer-engagement-domain-1779549600.json`
- `evidence/vcs/cs-ent-crm-customer-engagement-domain-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: CRM customer-engagement domain foundation. The domain
registers customer account metadata, qualifies opportunity metadata, prepares
quote metadata, opens service-case metadata, plans marketing-campaign metadata,
and records loyalty activity metadata with validation, data-class annotations,
false runtime/cloud flags, and Tenant RBAC ERP parity-map linkage. This
slice makes no durable customer master, CDP unification, CPQ price engine,
order-management mutation, service routing, knowledge-base integration,
marketing journey execution, message delivery, loyalty wallet settlement,
Workflow execution, runtime audit-chain emission, or cloud deployment claim.

Verification:

- `cargo check -p oya-crm-customer-engagement-domain`
- `cargo test --locked -p oya-crm-customer-engagement-domain`
- `cargo clippy --locked -p oya-crm-customer-engagement-domain --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-erp-parity-map`
- `cargo clippy --locked -p oya-tenant-rbac-erp-parity-map --all-targets -- -D warnings`
- 33-package enterprise regression test/clippy package group
- `cargo fmt --all -- --check`
- `jq . specs/microservices/crm.json`
- data-class annotation scan for CRM domain/spec/catalog
- quality-marker scan for CRM domain/spec/catalog
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-crm-customer-engagement-domain-1779549600.json --severity error`

### CS-ENT-SUPPLY-CHAIN-PLANNING-001 — Supply-chain planning domain foundation

Disjoint path envelope:

- `microservices/supply-chain-planning/crates/oya-supply-chain-planning-domain/**`
- `registry/catalog/oya-supply-chain-planning-domain.yaml`
- `specs/microservices/supply-chain-planning.json`
- `Cargo.toml`
- `Cargo.lock`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-erp-parity-map/**`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-supply-chain-planning-domain-1779549000.json`
- `evidence/vcs/cs-ent-supply-chain-planning-domain-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: SCM/APO supply-chain planning domain foundation. The domain
approves consensus demand-plan metadata, proposes supply-network plan metadata,
prepares available-to-promise response metadata, and prepares distribution-lane
plan metadata with validation, data-class annotations, false runtime/cloud flags,
and Tenant RBAC ERP parity-map linkage. This slice makes no durable planning
store, live demand-sensing ML, optimizer/scheduler/CTP runtime, production order,
procurement requisition, inventory mutation, warehouse reservation, carrier
booking, order-management rescheduling, Workflow execution, runtime audit-chain
emission, or cloud deployment claim.

Verification:

- `cargo check -p oya-supply-chain-planning-domain`
- `cargo test --locked -p oya-supply-chain-planning-domain`
- `cargo clippy --locked -p oya-supply-chain-planning-domain --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-erp-parity-map`
- `cargo clippy --locked -p oya-tenant-rbac-erp-parity-map --all-targets -- -D warnings`
- 32-package enterprise regression test/clippy package group
- `cargo fmt --all -- --check`
- `jq . specs/microservices/supply-chain-planning.json`
- data-class annotation scan for supply-chain-planning domain/spec/catalog
- quality-marker scan for supply-chain-planning domain/spec/catalog
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-supply-chain-planning-domain-1779549000.json --severity error`

## Honest-claim boundary

This plan can honestly claim only implemented domain invariants, metadata-only app-layer orchestration envelopes, API DTO contracts, preview OpenAPI wire-shape contracts, and passing targeted tests/gates. It does not claim production payroll compliance, statutory filing certification, GA ERP parity, measured SLOs, live connectors, durable audit emission, Workflow execution, deployed HTTP endpoints, or cloud integration until runtime/adapters/evidence gates exist.

### CS-ENT-GLOBAL-TRADE-001 — Global trade compliance domain foundation

Disjoint path envelope:

- `microservices/global-trade/crates/oya-global-trade-compliance-domain/**`
- `registry/catalog/oya-global-trade-compliance-domain.yaml`
- `specs/microservices/global-trade.json`
- `Cargo.toml`
- `Cargo.lock`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-erp-parity-map/**`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-global-trade-compliance-domain-1779550200.json`
- `evidence/vcs/cs-ent-global-trade-compliance-domain-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: GTS/global-trade compliance domain foundation. The domain
screens trade-party metadata, classifies trade-item metadata, assesses
export-control metadata, prepares customs-declaration metadata, and simulates
landed-cost metadata with validation, data-class annotations, false
runtime/cloud flags, and Tenant RBAC ERP parity-map linkage. This slice
makes no live denied-party screening, government list download, regulatory
content subscription, legal ruling, customs/export filing, broker network,
shipment/order/inventory/accounting mutation, Workflow execution, runtime
audit-chain emission, or cloud deployment claim.

Verification:

- `cargo check -p oya-global-trade-compliance-domain`
- `cargo test --locked -p oya-global-trade-compliance-domain`
- `cargo clippy --locked -p oya-global-trade-compliance-domain --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-erp-parity-map`
- `cargo clippy --locked -p oya-tenant-rbac-erp-parity-map --all-targets -- -D warnings`
- 34-package enterprise regression test/clippy package group
- `cargo fmt --all -- --check`
- `jq . specs/microservices/global-trade.json`
- data-class annotation scan for global-trade domain/spec/catalog
- quality-marker scan for global-trade domain/spec/catalog
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-global-trade-compliance-domain-1779550200.json --severity error`

### CS-ENT-REAL-ESTATE-001 — Real estate portfolio domain foundation

Disjoint path envelope:

- `microservices/real-estate/crates/oya-real-estate-portfolio-domain/**`
- `registry/catalog/oya-real-estate-portfolio-domain.yaml`
- `specs/microservices/real-estate.json`
- `Cargo.toml`
- `Cargo.lock`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-erp-parity-map/**`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-real-estate-portfolio-domain-1779550800.json`
- `evidence/vcs/cs-ent-real-estate-portfolio-domain-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: RE-FX real-estate portfolio domain foundation. The domain
registers property/rental-object metadata, registers lease-contract metadata,
projects lease cash-flow metadata, plans space-occupancy metadata, and prepares
facility-maintenance linkage metadata with validation, data-class annotations,
false runtime/cloud flags, and Tenant RBAC ERP parity-map linkage. This
slice makes no durable real-estate store, SAP RE-FX/SAP Cloud for Real Estate
integration, lease-accounting engine, GL/AP/AR posting, payment execution,
plant-maintenance work order, workspace/team sync, document archive, Workflow
execution, runtime audit-chain emission, or cloud deployment claim.

Verification:

- `cargo check -p oya-real-estate-portfolio-domain`
- `cargo test --locked -p oya-real-estate-portfolio-domain`
- `cargo clippy --locked -p oya-real-estate-portfolio-domain --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-erp-parity-map`
- `cargo clippy --locked -p oya-tenant-rbac-erp-parity-map --all-targets -- -D warnings`
- 35-package enterprise regression test/clippy package group
- `cargo fmt --all -- --check`
- `jq . specs/microservices/real-estate.json`
- data-class annotation scan for real-estate domain/spec/catalog
- quality-marker scan for real-estate domain/spec/catalog
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-real-estate-portfolio-domain-1779550800.json --severity error`

### CS-ENT-SUITE-CLOUD-DEPLOYMENT-MANIFEST-001 — Tenant RBAC cloud deployment manifest foundation

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-cloud-deployment-manifest/**`
- `registry/catalog/oya-tenant-rbac-cloud-deployment-manifest.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `specs/microservices/tenant-rbac.json`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-cloud-readiness-gate/**`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-cloud-deployment-manifest-1779551400.json`
- `evidence/vcs/cs-ent-platform-cloud-deployment-manifest-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: Tenant RBAC cloud deployment manifest foundation. The
manifest records Kubernetes namespace/deployment/service-account shape,
digest-pinned image policy, probes, replica/resource bounds, ArgoCD application
ref, Jenkins quality gate ref, Cosign policy ref, network policy ref, OTel
collector ref, SLO target, and explicit imperative-deploy refusal. The cloud
readiness gate consumes the manifest as a local pre-cloud artifact while keeping
ArgoCD controller, Kubernetes cluster, image publication, runtime Cosign
verification, runtime OTel export, cloud deployment evidence, production SLO
evidence, deployed listener, auth runtime, durable storage, Workflow/broker,
filing/disbursement rails, and runtime audit-chain emission as non-claims.

Verification:

- `cargo check -p oya-tenant-rbac-cloud-deployment-manifest`
- `cargo test --locked -p oya-tenant-rbac-cloud-deployment-manifest`
- `cargo clippy --locked -p oya-tenant-rbac-cloud-deployment-manifest --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-cloud-readiness-gate`
- `cargo clippy --locked -p oya-tenant-rbac-cloud-readiness-gate --all-targets -- -D warnings`
- 36-package enterprise regression test/clippy package group
- `cargo fmt --all -- --check`
- `jq . specs/microservices/tenant-rbac.json`
- data-class annotation scan for cloud manifest/readiness fields
- quality-marker scan for cloud manifest source/tests, spec, and catalog
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-cloud-deployment-manifest-1779551400.json --severity error`

### CS-ENT-SUITE-AUTH-RUNTIME-001 — Tenant RBAC auth runtime foundation

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-auth-app/**`
- `registry/catalog/oya-tenant-rbac-auth-app.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `specs/microservices/tenant-rbac.json`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-cloud-readiness-gate/**`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-auth-runtime-1779552000.json`
- `evidence/vcs/cs-ent-platform-auth-runtime-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: Tenant RBAC auth runtime foundation. The runtime records
deny-by-default route policy coverage for local HR, Payroll, Accounting, and
Tenant RBAC routes; validates issuer, audience, nonce, session age, tenant
isolation, route scopes, sensitive-data MFA/AAL2 requirements, and break-glass
audit requirements; and is consumed by the cloud-readiness gate as a local
pre-cloud artifact. It keeps OIDC signature verification, JWKS/provider
integration, durable session storage, deployed gateway enforcement, cloud
deployment evidence, production SLO evidence, and runtime audit-chain emission
as non-claims/blockers.

Verification:

- `cargo check -p oya-tenant-rbac-auth-app`
- `cargo test --locked -p oya-tenant-rbac-auth-app`
- `cargo clippy --locked -p oya-tenant-rbac-auth-app --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-cloud-readiness-gate`
- `cargo clippy --locked -p oya-tenant-rbac-cloud-readiness-gate --all-targets -- -D warnings`
- 37-package enterprise regression test/clippy package group
- `cargo fmt --all -- --check`
- `jq . specs/microservices/tenant-rbac.json`
- data-class annotation scan for auth runtime/readiness fields
- quality-marker scan for auth runtime source/tests, readiness source/tests, spec, and catalog
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-auth-runtime-1779552000.json --severity error`

### CS-ENT-SUITE-POSTGRES-RLS-STORAGE-001 — Tenant RBAC Postgres/RLS storage schema foundation

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-postgres-rls-storage/**`
- `registry/catalog/oya-tenant-rbac-postgres-rls-storage.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `specs/microservices/tenant-rbac.json`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-cloud-readiness-gate/**`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-postgres-rls-storage-1779552600.json`
- `evidence/vcs/cs-ent-platform-postgres-rls-storage-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: Tenant RBAC Postgres/RLS durable-storage schema
foundation. The plan records tenant-scoped tables for policy admissions, group
close rollups, cross-product Workflow plans, incident rollback plans, and ops
commands with tenant/idempotency primary keys, required payload/audit evidence
columns, ENABLE ROW LEVEL SECURITY, FORCE ROW LEVEL SECURITY, restrictive
current_setting-based tenant policies, no-delete append-only semantics, and
cloud-readiness-gate composition. It keeps runtime database connection,
migration application, live RLS verification, durable storage runtime, cloud
database, and runtime audit-chain emission as non-claims/blockers.

Verification:

- `cargo check -p oya-tenant-rbac-postgres-rls-storage`
- `cargo test --locked -p oya-tenant-rbac-postgres-rls-storage`
- `cargo clippy --locked -p oya-tenant-rbac-postgres-rls-storage --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-cloud-readiness-gate`
- `cargo clippy --locked -p oya-tenant-rbac-cloud-readiness-gate --all-targets -- -D warnings`
- 38-package enterprise regression test/clippy package group
- `cargo fmt --all -- --check`
- `jq . specs/microservices/tenant-rbac.json`
- data-class annotation scan for Postgres/RLS storage plan and readiness fields
- quality-marker scan for Postgres/RLS storage source/tests, readiness source/tests, spec, and catalog
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-postgres-rls-storage-1779552600.json --severity error`

### CS-ENT-SUITE-LISTENER-GATEWAY-001 — Tenant RBAC listener/gateway foundation

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-listener-gateway/**`
- `registry/catalog/oya-tenant-rbac-listener-gateway.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `specs/microservices/tenant-rbac.json`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-cloud-readiness-gate/**`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-listener-gateway-1779553200.json`
- `evidence/vcs/cs-ent-platform-listener-gateway-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: Tenant RBAC listener/gateway foundation. The plan
composes the 19-route local runtime catalog, 19-route auth policy, and cloud
deployment manifest into a review-only Kubernetes ClusterIP Service + Gateway
API HTTPRoute contract with TLS, network-policy, authz, route-scope, probe,
timeout, backend-port, and no-direct-public-NodePort/LoadBalancer requirements.
It keeps deployed listener runtime evidence, Gateway controller attachment,
load balancer provisioning, TLS certificate attachment, runtime auth
middleware, cloud deployment evidence, production SLO evidence, and runtime
audit-chain emission as non-claims/blockers.

Verification:

- `cargo check -p oya-tenant-rbac-listener-gateway`
- `cargo test --locked -p oya-tenant-rbac-listener-gateway`
- `cargo clippy --locked -p oya-tenant-rbac-listener-gateway --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-cloud-readiness-gate`
- `cargo clippy --locked -p oya-tenant-rbac-cloud-readiness-gate --all-targets -- -D warnings`
- 39-package enterprise regression test/clippy package group
- `cargo fmt --all -- --check`
- `jq . specs/microservices/tenant-rbac.json`
- data-class annotation scan for listener/gateway plan and readiness fields
- quality-marker scan for listener/gateway source/tests, readiness source/tests, spec, and catalog
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-listener-gateway-1779553200.json --severity error`

### CS-ENT-SUITE-IDP-VERIFICATION-001 — Tenant RBAC identity-provider verification foundation

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-identity-provider-verification/**`
- `registry/catalog/oya-tenant-rbac-identity-provider-verification.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `specs/microservices/tenant-rbac.json`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-cloud-readiness-gate/**`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-idp-verification-1779553800.json`
- `evidence/vcs/cs-ent-platform-idp-verification-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: Tenant RBAC identity-provider verification foundation.
The plan composes the auth runtime issuer/audience into a review-only OIDC
Discovery + JWKS + JWT-claim verification contract with TLS, issuer/audience,
expiry/not-before/issued-at, nonce, key id, tenant claim, subject claim,
MFA/assurance claim, route-scope alignment, asymmetric-algorithm, JWKS cache,
and key-rotation requirements. It keeps discovery fetch, JWKS fetch, OIDC
signature verification, external identity-provider attachment, token
introspection, durable session storage, runtime auth middleware, cloud gateway
enforcement, and runtime audit-chain emission as non-claims/blockers.

Verification:

- `cargo check -p oya-tenant-rbac-identity-provider-verification`
- `cargo test --locked -p oya-tenant-rbac-identity-provider-verification`
- `cargo clippy --locked -p oya-tenant-rbac-identity-provider-verification --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-cloud-readiness-gate`
- `cargo clippy --locked -p oya-tenant-rbac-cloud-readiness-gate --all-targets -- -D warnings`
- 40-package enterprise regression test/clippy package group
- `cargo fmt --all -- --check`
- `jq . specs/microservices/tenant-rbac.json`
- data-class annotation scan for identity-provider verification and readiness fields
- quality-marker scan for identity-provider verification source/tests, readiness source/tests, spec, and catalog
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-idp-verification-1779553800.json --severity error`

### CS-ENT-SUITE-AUDIT-CHAIN-EMISSION-001 — Tenant RBAC audit-chain emission contract foundation

Disjoint path envelope:

- `microservices/tenant-rbac/crates/oya-tenant-rbac-audit-chain-emission/**`
- `crates/oya-audit-chain-emission-kernel/src/lib.rs`
- `crates/oya-audit-chain-sealing-kernel/src/lib.rs`
- `registry/catalog/oya-tenant-rbac-audit-chain-emission.yaml`
- `registry/catalog/oya-tenant-rbac-{auth-runtime,cloud-deployment-manifest,cloud-readiness-gate,erp-parity-map,local-inmemory-harness,local-runtime-composition,postgres-rls-storage}.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `specs/microservices/tenant-rbac.json`
- `microservices/tenant-rbac/crates/oya-tenant-rbac-cloud-readiness-gate/**`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-audit-chain-emission-1779661200.json`
- `evidence/vcs/cs-ent-platform-audit-chain-emission-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: Tenant RBAC audit-chain emission contract foundation.
The plan defines CloudEvents-style context attributes, W3C traceparent
correlation, OpenTelemetry log-mapping intent, tenant/idempotency/payload-digest
extensions, source evidence references, nine tenant-scoped event schemas,
digest-only payload rules, WAL/outbox/Merkle prerequisites, and
cloud-readiness-gate composition. It keeps write-ahead-log runtime, broker
publish, Merkle sealing runtime, cloud audit sink, and runtime audit-chain
emission as non-claims/blockers while normalizing the prior Postgres/RLS
catalog rows and audit-chain kernel data_class annotations needed for honest
CI gate evidence.

Verification:

- `cargo check -p oya-tenant-rbac-audit-chain-emission`
- `cargo test --locked -p oya-tenant-rbac-audit-chain-emission`
- `cargo clippy --locked -p oya-tenant-rbac-audit-chain-emission --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-cloud-readiness-gate`
- `cargo clippy --locked -p oya-tenant-rbac-cloud-readiness-gate --all-targets -- -D warnings`
- 41-package enterprise regression test/clippy package group
- `cargo fmt --all -- --check`
- `jq . specs/microservices/tenant-rbac.json`
- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog` (currently advances past normalized Tenant RBAC rows, then blocks on pre-existing missing `oya-tenancy-tenant-lifecycle-kernel` catalog row)
- `./bin/oya gate validate data-class --workspace Cargo.toml` (currently advances past audit-chain kernel annotations, then blocks on pre-existing `RebalanceTask.from_cell`)
- quality-marker scan for audit-chain emission source/tests, readiness source/tests, spec, and catalog
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-audit-chain-emission-1779661200.json --severity error`

### CS-ENT-CI-GATE-UNBLOCKERS-001 — Tenancy catalog/data-class gate unblockers for enterprise verification

Disjoint path envelope:

- `registry/catalog/oya-tenancy-{tenant-lifecycle,isolation-policy,cell-assignment,dsr-cascade,sub-scope-registry,lifecycle-locks}-kernel.yaml`
- `registry/catalog/oya-tenancy-{reserved-namespace,dr-pairing,per-tenant-quota}-usecase.yaml`
- `registry/catalog/oya-tenancy-kyb-kyc-verifier-domain.yaml`
- `registry/catalog/oya-tenancy-data-residency-enforcer-adapter.yaml`
- `crates/oya-tenancy-{tenant-lifecycle,cell-assignment,isolation-policy,dsr-cascade,sub-scope-registry,lifecycle-locks}-kernel/src/lib.rs`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-ci-gate-unblockers-1779661800.json`
- `evidence/vcs/cs-ent-ci-gate-unblockers-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: enterprise verification gate hygiene needed before broad CI
claims. This slice adds missing catalog coverage for the tenancy workspace
crates that blocked `oya catalog validate` after the Tenant RBAC catalog
normalization, and annotates current tenancy kernel fields so
`oya gate validate data-class --workspace Cargo.toml` passes. It does not
change tenancy behavior, runtime isolation, DSR execution, lifecycle logic, or
Tenant RBAC cloud-readiness blockers. Catalog validation advances to the
next unrelated pre-existing missing record (`oya-payments-adapter-adyen`).

Verification:

- `cargo test --locked` for the affected tenancy packages
- `cargo clippy --locked` for the affected tenancy packages
- `cargo fmt --all -- --check`
- `./bin/oya gate validate data-class --workspace Cargo.toml`
- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog` (currently advances past the tenancy rows added here, then blocks on pre-existing missing `oya-payments-adapter-adyen`)
- quality-marker scan for touched tenancy kernels/catalog/evidence
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-ci-gate-unblockers-1779661800.json --severity error`

### CS-ENT-CI-GATE-CATALOG-COVERAGE-001 — Audit-chain/payments catalog coverage closure for enterprise verification

Disjoint path envelope:

- `registry/catalog/oya-audit-chain-{emission,query,retention-cascade,sealing,verification}-{api,domain,kernel}.yaml`
- `registry/catalog/oya-payments-*.yaml`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-ci-gate-catalog-coverage-1779662400.json`
- `evidence/vcs/cs-ent-ci-gate-catalog-coverage-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: close the remaining workspace catalog record coverage gaps
that block honest broad enterprise CI evidence after the tenancy gate cleanup.
The slice adds catalog rows for the audit-chain split scaffold crates and the
payments bounded-context scaffold crates, using conservative data-class labels
for payment-card scope and architecture-boundary-preserving roles for current
DTO-only audit-chain contract packages. It does not add PSP network calls,
payment execution, merchant onboarding runtime, settlement ingestion, audit
runtime storage, cloud deployment, or production compliance claims.

Verification:

- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog`
- `./bin/oya gate validate architecture-boundaries --repo-root . --registry registry/catalog --self-test`
- `./bin/oya gate validate data-class --workspace Cargo.toml`
- `cargo check -p` for the audit-chain split package group and payments package group
- `cargo clippy --locked -p` for the audit-chain split package group and payments package group
- `cargo fmt --all -- --check`
- quality-marker scan for touched catalog/tasks/evidence rows
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-ci-gate-catalog-coverage-1779662400.json --severity error`

### CS-ENT-CI-GATE-ARCH-ROLE-MATRIX-001 — Enterprise catalog role-matrix normalization

Disjoint path envelope:

- `registry/catalog/oya-{hr-employment,payroll-run,accounting-journal}-storage-adapter-inmemory.yaml`
- `registry/catalog/oya-tenant-rbac-{local-inmemory-harness,storage-adapter-inmemory,workflow-adapter-inmemory,identity-provider-verification,listener-gateway,cloud-readiness-gate}.yaml`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-ci-gate-arch-role-matrix-1779663000.json`
- `evidence/vcs/cs-ent-ci-gate-arch-role-matrix-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: reduce the remaining `architecture-boundaries` blocker by
making catalog roles match the actual dependency posture of local runtime
reference/harness/gate crates. The rows changed here are not pure clean-arch
adapters or tests: they compose app-layer envelopes, in-memory queues/stores,
auth/cloud/listener plans, or readiness inputs, so the repo's `runtime` role is
the honest matrix role for their current dependency edges. The slice does not
move package directories, does not change runtime behavior, and does not claim
production storage, listener, Workflow, PSP/payment, audit-chain runtime, cloud
deployment, or SLO evidence.

Verification:

- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog`
- `./bin/oya gate validate architecture-boundaries --repo-root . --registry registry/catalog` (expected to retain only pre-existing microservices/* package-placement diagnostics)
- `./bin/oya gate validate data-class --workspace Cargo.toml`
- affected HR/payroll/accounting/Tenant RBAC package-group check/test/clippy smoke where role rows changed
- `cargo fmt --all -- --check`
- quality-marker scan for touched catalog/tasks/evidence rows
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-ci-gate-arch-role-matrix-1779663000.json --severity error`

### CS-ENT-CI-GATE-PACKAGE-LAYOUT-001 — Enterprise microservice package layout normalization

Disjoint path envelope:

- `microservices/{hr,payroll,accounting,procurement,treasury,warehouse,production-planning,quality-management,plant-maintenance,supply-chain-planning,crm,global-trade,real-estate,tenant-rbac}/crates/**` moved to `crates/oya-*`
- `Cargo.toml`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-ci-gate-package-layout-1779663600.json`
- `evidence/vcs/cs-ent-ci-gate-package-layout-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: clear the remaining architecture-boundaries package-placement
blocker by moving the 41 enterprise microservice workspace packages from
`microservices/*/crates/oya-*` to the repository-standard `crates/oya-*` layout
and rewriting workspace/dependency path references. This is a layout-only
normalization: package names, source code, tests, catalog rows, runtime behavior,
cloud readiness claims, payment/statutory rails, durable storage, Workflow
execution, and SLO claims are unchanged.

Verification:

- `cargo metadata --format-version=1 --no-deps` shows the 41 enterprise packages under `crates/`
- manifest stale-path scan for `microservices/*/crates` and `../../../../crates` path dependencies
- `./bin/oya gate validate architecture-boundaries --repo-root . --registry registry/catalog`
- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog`
- `./bin/oya gate validate data-class --workspace Cargo.toml`
- moved package-group `cargo check`, `cargo test --locked`, and `cargo clippy --locked --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- quality-marker scan for touched manifests/tasks/evidence rows
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-ci-gate-package-layout-1779663600.json --severity error`
- audit-chain HEAD-prefix append-only check and Oya VCS claim/status/verify/done/promote transcript

### CS-ENT-SUITE-SLO-EVIDENCE-001 — Tenant RBAC SLO evidence contract foundation

Disjoint path envelope:

- `crates/oya-tenant-rbac-slo-evidence/**`
- `crates/oya-tenant-rbac-cloud-readiness-gate/{Cargo.toml,src/lib.rs,tests/readiness.rs}`
- `microservices/tenant-rbac/slos/*.openslo.yaml`
- `registry/catalog/oya-tenant-rbac-slo-evidence.yaml`
- `Cargo.toml`
- `Cargo.lock`
- `specs/microservices/tenant-rbac.json`
- `tasks/enterprise-microservices-*`
- `evidence/multispectrum/cs-ent-platform-slo-evidence-1779664200.json`
- `evidence/vcs/cs-ent-platform-slo-evidence-oya-vcs-lifecycle-20260524.json`
- `evidence/audit-chain.jsonl`

Acceptance focus: define a typed Tenant RBAC SLO evidence contract for later
Oyatie cloud promotion without overclaiming production SLO evidence. The slice
adds OpenSLO-style manifest paths, OTel metric stream names, rolling
error-budget windows, multi-window burn-rate alert policy, canary evidence refs,
rollback release-gate refs, and cloud-readiness composition. It keeps runtime
OTel export, metrics backend attachment, alert-manager attachment, canary
runtime, rollback automation, production SLO evidence, multi-region SLO
evidence, cloud deployment, deployed listener, durable storage, Workflow
execution, statutory filing, disbursement rails, and runtime audit-chain
emission as blockers/non-claims.

Verification:

- `cargo check -p oya-tenant-rbac-slo-evidence`
- `cargo test --locked -p oya-tenant-rbac-slo-evidence`
- `cargo clippy --locked -p oya-tenant-rbac-slo-evidence --all-targets -- -D warnings`
- `cargo test --locked -p oya-tenant-rbac-cloud-readiness-gate`
- `cargo clippy --locked -p oya-tenant-rbac-cloud-readiness-gate --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `jq . specs/microservices/tenant-rbac.json`
- OpenSLO manifest parse/shape scan for `microservices/tenant-rbac/slos/*.openslo.yaml`
- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog`
- `./bin/oya gate validate architecture-boundaries --repo-root . --registry registry/catalog`
- `./bin/oya gate validate data-class --workspace Cargo.toml`
- quality-marker scan for touched SLO evidence/readiness/spec/task/evidence rows
- `./bin/oya gate validate dependency-seam --evidence evidence/multispectrum/cs-ent-platform-slo-evidence-1779664200.json --severity error`
- audit-chain HEAD-prefix append-only check and Oya VCS claim/status/verify/done/promote transcript
