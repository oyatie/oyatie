---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-first-paying-tenant
phase: P02-payroll
status: Proposed
acceptance_lanes: []
entry_gate: |
  M03/P01-hr complete; oya-hr-employee-kernel + oya-hr-employment-kernel ship;
  Employee/Employment Object Types registered in Ontology;
  oya-finance-library-domain ships (M02-P11 substrate);
  EmployeeHired Workflow event registered.
exit_gate: |
  All IP acceptance gates green; `cargo nextest run -p oya-payroll-*` 0 failures;
  `oya gate validate lean-a2 --ms payroll` exits 0;
  `oya gate validate audit-chain --ms payroll` exits 0;
  4대보험 EDI format compliance test green;
  연말정산 21-category test green;
  k6 smoke payroll run 1k employees ≤30s; payslip p99 ≤50ms;
  grit done on all P02 symbols; ICM phase-handoff row emitted.
depends_on:
  - milestone: M03
    phase: P01-hr
    reason: "Payroll reads Employee/Employment from Ontology; EmployeeHired event triggers payroll enrollment; requires HR to ship first."
  - milestone: M02
    phase: P11-finance-library
    reason: "oya-finance-library-domain provides Money + KRW arithmetic used in gross-to-net engine."
parallel_wave: 2
owner_team: council-enterprise
---

# P02-payroll: Payroll µservice — KR gross-to-net, 4대보험 EDI, 연말정산, disbursement

## Purpose

Delivers the `oya-payroll-*` µservice: monthly payroll computation (gross-to-net
under Korean tax law), 4대보험 EDI electronic submission (NPS / NHIS / MOEL),
연말정산 year-end settlement, payslip generation (Typst PDF), and bank disbursement
batch files. Consumes Employee/Employment from Ontology; emits `PayrollRunCompleted`
and `DisbursementBatchReady` to Workflow for Accounting and Connect downstream.

Advances ADR-0210 M3 KR group payroll launch scope: at least one paid KR group
customer closes real payroll before M3 is declared complete. This phase is the
computational core that makes that closure possible.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Crate family (BNF v4.1) |
|---|---|---|
| `payroll` | `run` | `oya-payroll-run-{kernel,domain,application,adapter,rest}` |
| `payroll` | `insurance` | `oya-payroll-insurance-{kernel,domain,application,adapter}` |
| `payroll` | `year-end` | `oya-payroll-year-end-{kernel,domain,application,adapter,rest}` |
| `payroll` | `payslip` | `oya-payroll-payslip-{kernel,domain,application,adapter,rest}` |
| `payroll` | `disbursement` | `oya-payroll-disbursement-{kernel,domain,application,adapter}` |
| `payroll` | `app` | `oya-payroll-app` |

Naming justifications:

```
NAME: oya-payroll-run-kernel
JUSTIFICATION:
- microservice = payroll: Payroll µservice; registered; ADR-0056 v4.1
- bc-tokens = run: payroll has multiple BCs (run/insurance/year-end/payslip/disbursement); run BC owns PayrollRun entity + PayrollEntry + PayrollRunRepository port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure RunId/EntryId value types + PayrollRunRepository port-trait; zero logic; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-payroll-run-domain
JUSTIFICATION:
- microservice = payroll; bc-tokens = run; layer = domain: gross-to-net computation engine + pro-rata logic + deduction rules (4대보험 rates, income tax withholding per EmploymentClassification); calls through PayrollRunRepository; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-payroll-insurance-kernel
JUSTIFICATION:
- microservice = payroll; bc-tokens = insurance: insurance BC owns InsuranceConfig entity + EdiSubmission aggregate + EdiSubmissionRepository port-trait; 4대보험 (NPS/NHIS/고용/산재) EDI adapter ports; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure types + port declarations; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-payroll-year-end-kernel
JUSTIFICATION:
- microservice = payroll; bc-tokens = year-end: year-end BC owns YearEndSettlement entity + WithholdingCert + 21-category deduction model; YearEndRepository port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure types + port declarations; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-payroll-payslip-kernel
JUSTIFICATION:
- microservice = payroll; bc-tokens = payslip: payslip BC owns Payslip entity + Typst render port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure types + PayslipRenderer port declaration; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-payroll-disbursement-kernel
JUSTIFICATION:
- microservice = payroll; bc-tokens = disbursement: disbursement BC owns DisbursementBatch entity + bank-transfer format port-trait (KR CMS/NEMS); ADR-0056 v4.1 BC-optionality
- layer = kernel: pure types + DisbursementStore/BankTransferPort declarations; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-payroll-app
JUSTIFICATION:
- microservice = payroll; bc-tokens: OMITTED — composition-root; ADR-0056 §"BC optionality"
- layer = app: main.rs + DI wiring for all payroll BCs; ADR-0056 §"Layer semantics"
- exemptions: none
```

### Out-of-scope

- Direct statutory agency EDI submission (automated NPS/NHIS file upload) — deferred post-M03 per ADR-0210 §"Explicitly Post-M3".
- Full prior-year 연말정산 recomputation/backfill — deferred post-M03; M03 ships signed-fact import path only.
- Non-KR jurisdiction payroll — deferred to M04+ per ADR-0210.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full payroll µservice scaffold: DDL, gross-to-net engine, 4대보험 EDI adapters, 연말정산, Typst payslip, bank disbursement batch, Workflow events, Ontology writes, load tests | pending | council-enterprise |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features                                # exit 0
cargo build -p oya-payroll-app --all-features                         # exit 0
cargo clippy -p oya-payroll-run-domain -p oya-payroll-insurance-domain -p oya-payroll-year-end-domain -- -D warnings  # exit 0
cargo nextest run -p oya-payroll-run-domain --test payroll_run_1000   # exit 0; ≤30s for 1000 employees
cargo nextest run -p oya-payroll-insurance-domain --test edi_format_compliance  # exit 0; 더존 reference match
cargo nextest run -p oya-payroll-year-end-domain --test year_end_all_categories  # exit 0; all 21 deduction categories
cargo nextest run -p oya-payroll-payslip-domain                       # exit 0
cargo nextest run -p oya-payroll-disbursement-domain                  # exit 0
cargo deny check                                                      # exit 0
```

### Fitness lane gates

```bash
oya gate validate lean-a2 --ms payroll          # no imports from hr/accounting/connect
oya gate validate lean-a1 --ms payroll          # layer ordering
oya gate validate port-location --ms payroll    # port traits in kernel
oya gate validate shardability --ms payroll     # tenant_id partition key
oya gate validate audit-chain --ms payroll      # Ed25519 seal per (tenant, year, month)
oya gate validate jurisdiction-overlay --ms payroll  # jurisdiction_code=KR
```

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --ms payroll  # PayrollRunCompleted/PayrollFinalizedForEmployee/EdiSubmissionQueued/YearEndSettlementCompleted/DisbursementBatchReady
oya gate validate ontology-type-registry --ms payroll   # PayrollEntry/WithholdingCert/Payslip Object Types
```

### Performance gates

```bash
# k6: payslip read p99 ≤50ms at 1k RPS
k6 run tests/load/smoke-payroll-payslip-read.js --env BASE_URL=http://localhost:8082
# Pass: http_req_duration{p(99)}<50; error rate <0.1%

# Integration: payroll run 1k employees ≤30s
cargo nextest run -p oya-payroll-run-domain --test payroll_run_1000 --no-fail-fast
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-payroll-run-kernel` | `kernel` | Yes — `PayrollRunRepository`, `PayrollEntryStore` | N/A |
| `oya-payroll-run-domain` | `domain` | N/A | N/A |
| `oya-payroll-run-application` | `application` | N/A | N/A |
| `oya-payroll-run-adapter` | `adapter` | N/A | Yes — `PostgresPayrollRunRepository`, `OntologyPayrollEntryWriter` |
| `oya-payroll-insurance-kernel` | `kernel` | Yes — `EdiSubmissionPort`, `InsuranceConfigStore` | N/A |
| `oya-payroll-insurance-adapter` | `adapter` | N/A | Yes — `NpsEdiAdapter`, `NhisEdiAdapter`, `MoelEdiAdapter` |
| `oya-payroll-year-end-kernel` | `kernel` | Yes — `YearEndRepository`, `WithholdingCertRenderer` | N/A |
| `oya-payroll-payslip-kernel` | `kernel` | Yes — `PayslipRenderer` (Typst port) | N/A |
| `oya-payroll-payslip-adapter` | `adapter` | N/A | Yes — `TypstPayslipRenderer` |
| `oya-payroll-disbursement-kernel` | `kernel` | Yes — `BankTransferPort` (CMS/NEMS) | N/A |
| `oya-payroll-disbursement-adapter` | `adapter` | N/A | Yes — `CmsBatchAdapter`, `NemsBatchAdapter` |
| `oya-payroll-app` | `app` | N/A | Unrestricted inward |

Cross-product integration: payroll NEVER imports `oya-hr-*`. Employee/Employment
read via `oya-ontology-entity-kernel::ObjectStore` port only.

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `run` | `payroll` | pending |
| `insurance` | `payroll` | pending |
| `year-end` | `payroll` | pending |
| `payslip` | `payroll` | pending |
| `disbursement` | `payroll` | pending |

---

## Grit Claim Symbols

```
crates/oya-payroll-run-kernel/src/ports.rs::PayrollRunRepository
crates/oya-payroll-run-domain/src/gross_to_net.rs::GrossToNetEngine
crates/oya-payroll-insurance-kernel/src/ports.rs::EdiSubmissionPort
crates/oya-payroll-insurance-domain/src/edi_format.rs::FourInsuranceEdi
crates/oya-payroll-year-end-domain/src/deduction_categories.rs::YearEndDeductionCategory
crates/oya-payroll-disbursement-kernel/src/ports.rs::BankTransferPort
contracts/payroll.openapi.yaml::runPayroll
contracts/payroll.openapi.yaml::submitEdi
docs/standards/bounded-contexts.md::payroll.run
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P02-payroll started; depends on P01-hr complete + M02 finance-library; scope: 5 BCs gross-to-net/EDI/year-end/payslip/disbursement" \
  -i high \
  -k "M03,P02,phase-start,payroll"

icm store \
  -t context-oyatie \
  -c "Phase P02-payroll complete; KR payroll engine shipped; 4대보험 EDI (NPS/NHIS/MOEL) green; 연말정산 21 categories; Typst payslips; CMS bank batch; Workflow events PayrollRunCompleted/DisbursementBatchReady; next: P03-accounting" \
  -i high \
  -k "M03,P02,phase-complete,payroll"
```

---

## References

- PRD: `docs/prds/payroll.md`
- Bominal ADRs inherited: ADR-0210 (M3 scope), ADR-0126 (8-class employment), ADR-0028 (audit chain), ADR-0018 (tenancy RLS)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- Memory: `feedback_clean_architecture_requirements.md`, `feedback_workflow_objectgraph_adapter_layer.md`
