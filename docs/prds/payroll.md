---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-payroll
microservice: payroll
status: Accepted
sales_segment: Enterprise
tier: B2B
milestone_first_ship: M03-first-paying-tenant
bominal_source:
  - ADR-0210  # M03 KR group payroll + mail launch
  - ADR-0126  # employment classification (8 classes)
  - ADR-0125  # domain naming canon
  - ADR-0028  # audit chain Merkle/Ed25519
  - ADR-0018  # tenancy RLS posture
doc_status: published
---

# PRD-payroll: Payroll µservice

---

## Purpose

The Payroll µservice computes and disburses employee compensation for Korean
corporate tenants. M03 scope targets the KR group payroll launch per Bominal
ADR-0210: monthly payroll calculation, 4대보험 EDI submission, and 연말정산
(year-end tax settlement). It consumes employee/employment data from HR via
Ontology and emits payroll events to Accounting via Workflow.

Inherits from Bominal ADR-0210 (M03 KR group payroll + mail launch) 1:1.
No Bominal overrides.

---

## Tenant Value

- **KR-compliant payroll computation**: gross-to-net calculation under Korean
  tax law; deductions for 국민연금, 건강보험, 고용보험, 산재보험 (4대보험);
  income tax and local tax withholding.
- **4대보험 EDI**: automated electronic submission to NPS/NHIS/MOEL;
  eliminates manual portal entry.
- **연말정산**: year-end tax settlement with employee deduction declaration
  collection and finalized withholding certificates (근로소득원천징수영수증).
- **Payslip generation**: Typst-rendered payslips delivered to employees via
  Connect-Pro or downloadable PDF.
- **Accounting integration**: payroll journal entries written automatically to
  Accounting via Ontology; no double-entry by HR staff.

---

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | Payroll admin | run monthly payroll for all active employees in a payroll group | salaries are computed and queued for disbursement | `payroll-run` | Must |
| FR-02 | Payroll admin | configure 4대보험 rates and EDI submission credentials per tenant | EDI submissions go to the correct insurer endpoints | `insurance` | Must |
| FR-03 | Payroll admin | submit 4대보험 EDI files (취득/상실/변경/보수월액) to NPS/NHIS/MOEL | regulatory filings are automated; no manual portal entry | `insurance` | Must |
| FR-04 | Payroll admin | run 연말정산 settlement for all employees in January | finalized withholding certificates generated; refund/additional-tax amounts computed | `year-end` | Must |
| FR-05 | Employee | view my payslip for any month | I can verify my net pay breakdown | `payslip` | Must |
| FR-06 | Payroll admin | handle mid-month hires, terminations, and classification changes with pro-rata calculation | payroll is accurate for partial-month scenarios | `payroll-run` | Must |
| FR-07 | Payroll admin | export payroll data to banking system for mass transfer | disbursement batch sent to KB/신한/우리/하나 etc. | `disbursement` | Must |
| FR-08 | Auditor | access full payroll audit trail per employee per period | labor authority or tax authority investigations satisfied | `audit` | Must |

---

## Non-Functional Requirements

### Performance
- P99 payroll run for 1,000 employees: ≤30 s.
- P99 payslip read: ≤50 ms (Ontology Function).
- P99 EDI submission acknowledgment: ≤5 s (network-bound; async with status polling).

### Security
- JWT `tenant_id` enforced; payroll data never cross-tenant (ADR-0018).
- 4대보험 EDI credentials stored in tenant-scoped KMS (oyatie KMS µservice).
- Bank disbursement API keys: KMS-wrapped; audit log on every access.
- Cedar policy: payroll admins scoped to their org; employees read own payslips only.

### Audit + Compliance
- Every payroll run immutably sealed with Merkle/Ed25519 per (tenant_id, year, month)
  per ADR-0028; seal latency ≤1 s.
- 근로소득원천징수영수증 retention: 5 years per Korean tax law.
- EDI submission receipts stored immutably; resubmission creates new record (no overwrite).
- Jurisdiction overlay `KR` applied per ADR-0127.

### Availability + SLO
- 99.9% monthly. Payroll run deadline windows (end of month) treated as
  high-priority; runbook escalation path documented.
- RTO ≤30 s per-cell; RPO ≤5 s.

---

## Bounded Contexts

| BC name | Crate family (BNF v4.1) | Purpose | Key entities |
|---|---|---|---|
| `payroll-run` | `payroll-run-{domain,application,infrastructure,rest}` | Monthly payroll computation; gross-to-net; pro-rata | `PayrollRun`, `PayrollEntry` |
| `insurance` | `payroll-insurance-{domain,application,infrastructure}` | 4대보험 rate config; EDI submission; NPS/NHIS/MOEL adapters | `InsuranceConfig`, `EdiSubmission` |
| `year-end` | `payroll-year-end-{domain,application,infrastructure,rest}` | 연말정산 calculation; deduction collection; withholding cert gen | `YearEndSettlement`, `WithholdingCert` |
| `payslip` | `payroll-payslip-{domain,application,infrastructure,rest}` | Payslip generation (Typst); employee-facing read | `Payslip` |
| `disbursement` | `payroll-disbursement-{domain,application,infrastructure}` | Bank transfer batch; disbursement status tracking | `DisbursementBatch` |

```
NAME: payroll-run-domain
JUSTIFICATION:
- microservice = payroll: Payroll µservice; flat catalog; ADR-0056 v4.1
- bc-tokens = run: payroll has multiple BCs (run/insurance/year-end/payslip/disbursement); run BC owns the monthly computation cycle; ADR-0056 v4.1 BC-optionality
- layer = domain: PayrollRun entity + gross-to-net computation rules + PayrollEntryRepository port-trait; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none
```

---

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `PayrollRunCompleted` | All entries calculated; ready for disbursement | `accounting`, `payslip` | `payroll-run-sm` |
| `PayrollFinalizedForEmployee` | Individual entry finalized | `hr` (offboarding gate) | `payroll-run-sm` |
| `EdiSubmissionQueued` | Payroll run complete; EDI files ready | `insurance` (EDI adapter) | `edi-submission-sm` |
| `YearEndSettlementCompleted` | 연말정산 finalized for all employees | `accounting`, `payslip` | `year-end-sm` |
| `DisbursementBatchReady` | Payroll approved; bank transfer batch generated | `disbursement` | `disbursement-sm` |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `EmployeeHired` | `hr` | `payroll-run` | Register employee in next payroll cycle |
| `EmploymentClassChanged` | `hr` | `payroll-run` | Recalculate deductions from effective date |
| `OffboardingInitiated` | `hr` | `payroll-run` | Compute final paycheck; pro-rata calculation |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `PayrollEntry` | `BelongsToRun` → `PayrollRun` | `payroll-run` | Ed25519 per entry |
| `WithholdingCert` | `IssuedTo` → `Employee` | `year-end` | Ed25519 |
| `Payslip` | `ForPeriod` → `PayrollRun` | `payslip` | Ed25519 |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Employee` | `payroll-run` | `filter(tenant_id).where(active=true)` |
| `Employment` | `payroll-run` | `filter(tenant_id, employee_id)` — classification + salary |
| `Department` | `payroll-run` | `filter(tenant_id)` — cost-center rollup |

---

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| 더존비즈온 | 급여/4대보험 | 4대보험 EDI format (취득/상실/변경/보수월액); 연말정산 computation model; KR bank transfer format | https://www.douzone.com |
| ADP | ADP Workforce Now | Payroll run orchestration; gross-to-net engine; multi-jurisdiction; audit trail depth | https://www.adp.com |
| Gusto | Gusto Payroll | Payslip UX; onboarding-to-first-paycheck flow; employee self-service | https://gusto.com |
| 인크루트/잡코리아 HR | KR HR SaaS | KR-specific deduction field completeness | Market research |

Key parity gaps:
1. **4대보험 EDI format v5.0** (더존 reference): full `취득/상실/변경/보수월액` file format per NHIS/NPS schema — must be exact-match or EDI submissions fail.
2. **연말정산 deduction categories** (2025 tax year): 소득공제, 세액공제, 기부금, 의료비, 교육비, 주택관련 — full 21-category support.
3. **Bank transfer formats**: 국내 은행 CMS 파일 형식 (KB/신한/우리/하나 등) — batch file format compliance required before disbursement.

---

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Payroll run 1,000 employees | 5 s | 30 s | 60 s | Streaming; per-employee parallelism |
| Payslip read (Ontology Function) | 5 ms | 50 ms | 100 ms | ADR-0107 ≤50ms p99 |
| EDI submission ack | — | 5 s | — | Network-bound; async |
| Year-end settlement 1,000 employees | 10 s | 60 s | 120 s | |
| Audit chain seal per run | — | 1 s | — | Per (tenant_id, year, month); ADR-0028 |

Error budget: 0.1% monthly. SLO burn-rate alarm: 5×.

---

## Horizontal Scalability

**State strategy**: `postgres` — payroll entries, EDI submissions, withholding
certs in Postgres + Citus; `tenant_id` partition key; Postgres RLS.

**Active-active compatibility**: `single-writer-compatible` — payroll run is a
serialized transaction per (tenant, period); single writer per period to
prevent double-payment.

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Max employees per payroll run | 5,000 | 100,000 | Worker queue depth > 500 |
| Max QPS (payslip reads) | 1,000 | 10,000 | CPU > 70% |
| Max concurrent payroll runs | 10 | 100 | Worker pool exhaustion |

Scale-out: worker layer HPA on queue depth >500; min 2; max 50 worker pods.
Payroll run workers are stateless; state in Postgres only.
Cross-region: M03 KR only; post-M03 per `docs/ROADMAP.md`.

---

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Payroll run for 1,000 employees completes in ≤30 s; all entries correct | `cargo nextest run -p payroll-run-domain --test payroll_run_1000` |
| AC-02 | 4대보험 EDI file generated matches 더존 iCUBE reference format | `cargo nextest run -p payroll-insurance-domain --test edi_format_compliance` |
| AC-03 | 연말정산 all 21 deduction categories computed correctly | `cargo nextest run -p payroll-year-end-domain --test year_end_all_categories` |
| AC-04 | `PayrollRunCompleted` event routed to accounting | integration test `test_payroll_accounting_workflow` |
| AC-05 | LEAN-A2: no direct imports from hr/accounting/connect | `presubmit` (retired CLI `gate validate lean-a2 --ms payroll`) exits 0 |
| AC-06 | Payslip p99 ≤50 ms at 1k RPS | k6 smoke; `http_req_duration{p(99)}<50` |
| AC-07 | Audit chain sealed; tamper-evident per (tenant, year, month) | `presubmit` (retired CLI `gate validate audit-chain --ms payroll`) |

---

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | NPS/NHIS EDI endpoint: sandbox available for M03 integration tests? | payroll-team | M03/P01 |
| 2 | Bank transfer format priority order (KB first, or all 4 simultaneously)? | council-product | M03/P02 |
| 3 | 연말정산: deduction declaration collected via Connect-Pro form or external upload? | council-product | M03/P03 |

---

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0210 | M03 KR group payroll + mail launch | inherited — M03 scope definition |
| Bominal ADR-0126 | Employment classification | inherited — 8-class enum maps to deduction rules |
| Bominal ADR-0028 | Audit chain Merkle/Ed25519 | inherited |
| Bominal ADR-0018 | Tenancy RLS posture | inherited |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0103 | Workflow hexagonal | integration plane |
| ADR-0106 | Ontology architecture | information plane |
