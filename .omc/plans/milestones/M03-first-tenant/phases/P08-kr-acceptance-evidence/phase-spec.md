---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-first-paying-tenant
phase: P08-kr-acceptance-evidence
status: Proposed
acceptance_lanes: []
entry_gate: "M03/P06-application-b2b-live complete (B2B shell live; all \xB5services\
  \ enabled);\nM03/P07-workflow-studio-editor complete (Studio live; 10 domain templates\
  \ loaded);\nAt least one KR group paying tenant signed and onboarded in production\
  \ cell (OCI ap-seoul-1).\n"
exit_gate: "1 KR group paying tenant has closed real payroll in production (Bominal\
  \ ADR-0210 closure criterion);\n4\uB300\uBCF4\uD5D8 EDI monthly report green (NPS/NHIS/MOEL\
  \ submissions acknowledged);\n\uC5F0\uB9D0\uC815\uC0B0 audit-chain segment sealed\
  \ (\uADFC\uB85C\uC18C\uB4DD\uC6D0\uCC9C\uC9D5\uC218\uC601\uC218\uC99D generated\
  \ + Ed25519-sealed);\nLegal hold initiated + eDiscovery export verified (PST/MBOX\
  \ with chain of custody);\nSLO burn-rate \u2264 budget for 7 consecutive days (error\
  \ budget: \u22640.1% per \xB5service);\nRestore drill complete (tenant/entity-scoped;\
  \ M03 load test for 3,000-person shape);\ngrit done on all P08 symbols; ICM M03-close\
  \ row emitted.\n"
depends_on:
- milestone: M03
  phase: P06-application-b2b-live
  reason: Application B2B shell must be live for the KR group tenant to onboard and
    enable products.
- milestone: M03
  phase: P07-workflow-studio-editor
  reason: Workflow Studio must be live for tenant to run payroll-close workflow and
    configure automations.
parallel_wave: 5
owner_team: council-architecture
purpose: Auto-backfilled purpose for phase-spec.md
---
# P08-kr-acceptance-evidence: M3 KR group customer onboarding + acceptance evidence — payroll close, EDI green, 연말정산, legal hold, 7-day SLO

## Purpose

Closes the M03 milestone by producing the production evidence required by
Bominal ADR-0210 §"Launch-Blocking Client Bar" and §"M3 In Scope": one paid KR
group customer running real payroll in production, 4대보험 EDI submissions
acknowledged, 연말정산 audit chain sealed, legal hold initiated and eDiscovery
export verified, and 7 consecutive days of SLO burn-rate within budget.

This phase produces no new µservice code. It is an evidence-gathering and
acceptance-verification phase: load tests, restore drills, SLO monitoring,
and final audit-chain verification for all M03 µservices.

---

## Scope

### In-scope

| Activity | Artifact | Acceptance criterion |
|---|---|---|
| KR group tenant production onboarding | Tenant activation record + first payroll run ID | Payroll close confirmed by tenant admin in production |
| 4대보험 EDI monthly submission | NPS/NHIS/MOEL acknowledgment receipts (immutable; EDI submission receipts stored per PRD-payroll) | All three EDI endpoints return acknowledgment; stored in `oya-payroll-insurance-adapter` |
| 연말정산 audit-chain segment seal | Ed25519-sealed `YearEndSettlement` + `WithholdingCert` records per (tenant_id, year) | `oya gate validate audit-chain --ms payroll` exits 0 for year-end segment |
| Legal hold + eDiscovery evidence | `LegalHold` record initiated; PST/MBOX export sealed with chain of custody | `test_legal_hold_export_100k` passes in production cell |
| 7-day SLO burn-rate monitoring | Prometheus + Grafana dashboards per µservice | burn-rate ≤5× for all M03 µservices over 7 consecutive days |
| 3,000-person load test | k6 load test at 3k-employee payroll run | p99 payroll run ≤30s; p99 payslip read ≤50ms; p99 shell frame ≤100ms |
| Restore drill | Full tenant/entity-scoped restore from backup | RTO ≤30s; RPO ≤5s verified |
| KR regulatory corpus citation audit | All `LegalCitation { article_id, corpus_sha }` values in payroll/HR/accounting resolve against pinned corpus.lock | `oya gate validate corpus-citations --ms hr --ms payroll --ms accounting` exits 0 |

### Out-of-scope

- Post-M03 direct statutory agency submission (automated NPS/NHIS portal upload) — deferred per ADR-0210.
- Additional KR group tenants beyond the first — M04 scale-out work.
- Non-KR jurisdiction tenants — post-M03.
- Full prior-year 연말정산 backfill — post-M03.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | KR acceptance evidence: production load-test scripts for 3k-employee shape, SLO burn-rate monitoring setup, restore drill runbook, corpus citation audit gate, final audit-chain verification commands, grit claim + done ceremony | pending | council-architecture |

---

## Acceptance Gates

### M03 closure criterion (ADR-0210)

```bash
# 1. KR group paying tenant has closed real payroll in production
#    Evidence: payroll run_id + tenant_id from production Postgres; confirmed by tenant admin
oya gate validate m03-payroll-closure --tenant-id <production-tenant-id>  # exit 0

# 2. 4대보험 EDI green
oya gate validate edi-acknowledgment --ms payroll --period <YYYY-MM>  # exit 0; NPS+NHIS+MOEL ack receipts present

# 3. 연말정산 audit-chain segment sealed
oya gate validate audit-chain --ms payroll --segment year-end --year <YYYY>  # exit 0; Ed25519 sealed

# 4. Legal hold initiated + eDiscovery export verified
oya gate validate legal-hold-evidence --ms connect --tenant-id <production-tenant-id>  # exit 0; PST/MBOX with chain of custody

# 5. 7-day SLO burn-rate ≤ budget
oya gate validate slo-burn-rate --ms hr --ms payroll --ms accounting --ms connect --ms application --ms workflow --days 7  # exit 0; burn-rate ≤5× (0.1% budget)
```

### Load test gates (3,000-person shape)

```bash
# Payroll run 3k employees ≤30s
k6 run tests/load/m03-payroll-3k.js --env BASE_URL=https://production.oyatie.io
# Pass: payroll_run_duration{p(99)}<30000ms; error_rate<0.001

# Payslip read p99 ≤50ms at 1k RPS (3k employees × concurrent reads)
k6 run tests/load/m03-payslip-read-3k.js --env BASE_URL=https://production.oyatie.io
# Pass: http_req_duration{p(99)}<50; error_rate<0.001

# Shell frame p99 ≤100ms at 10k concurrent sessions
k6 run tests/load/m03-shell-10k.js --env BASE_URL=https://production.oyatie.io
# Pass: http_req_duration{p(99)}<100; error_rate<0.001

# Workflow Studio: 10k concurrent active runs p99 ≤200ms
k6 run tests/load/workflow-engine-10k.js --env BASE_URL=https://production.oyatie.io
```

### Restore drill gate

```bash
# Tenant/entity-scoped restore from backup
oya gate validate restore-drill --tenant-id <production-tenant-id> --target-rto 30s --target-rpo 5s
# Pass: RTO ≤30s; RPO ≤5s; data integrity checksum matches pre-restore state
```

### Corpus citation audit gate

```bash
# All LegalCitation { article_id, corpus_sha } values resolve against pinned corpus.lock
oya gate validate corpus-citations --ms hr --ms payroll --ms accounting
# Pass: all citations resolve; no expired corpus_sha; exit 0
```

### Audit-chain final verification (all M03 µservices)

```bash
oya gate validate audit-chain --ms hr
oya gate validate audit-chain --ms payroll
oya gate validate audit-chain --ms accounting
oya gate validate audit-chain --ms connect
oya gate validate audit-chain --ms application
oya gate validate audit-chain --ms workflow
# All: exit 0; tamper-detection passes
```

---

## Grit Claim Symbols

```
tests/load/m03-payroll-3k.js::payrollRun3kShape
tests/load/m03-payslip-read-3k.js::payslipRead3kShape
tests/load/m03-shell-10k.js::shellFrame10kSessions
docs/evidence/m03-kr-acceptance-bundle.json::payrollClosureEvidence
docs/evidence/m03-kr-acceptance-bundle.json::ediAcknowledgmentEvidence
docs/evidence/m03-kr-acceptance-bundle.json::yearEndAuditChainEvidence
docs/evidence/m03-kr-acceptance-bundle.json::legalHoldEvidenceExport
docs/evidence/m03-kr-acceptance-bundle.json::sloMonitoringEvidence
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P08-kr-acceptance-evidence started; all M03 µservices live; production KR group tenant onboarding; executing 3k-person load tests, EDI submission, 연말정산, legal hold, 7-day SLO monitoring" \
  -i critical \
  -k "M03,P08,phase-start,kr-acceptance,production"

icm store \
  -t context-oyatie \
  -c "M03 COMPLETE — P08-kr-acceptance-evidence closed: 1 KR group paying tenant live; payroll closed in production; 4대보험 EDI green; 연말정산 audit-chain sealed; legal hold + eDiscovery verified; SLO burn-rate ≤budget for 7 days; restore drill passed; corpus citations valid. M03-first-paying-tenant milestone declared complete." \
  -i critical \
  -k "M03,P08,phase-complete,m03-close,kr-acceptance,production"
```

---

## ADR-0210 M3 Closure Claim

Upon P08 exit gate passing, the following public claim is authorized (per ADR-0210):

> Oyatie supports production KR group payroll for employees and non-employee
> payees, with entity-level payroll close, Korean-first self-service, statutory
> export evidence, immutable ledgers, and expert-reviewed compliance rulepacks.

Mail public claim requires separate gate (per ADR-0210): Connect Professional
Mail acceptance evidence is included in the legal hold / eDiscovery gate above.

---

## References

- PRD: `docs/prds/hr.md`, `docs/prds/payroll.md`, `docs/prds/accounting.md`, `docs/prds/connect.md`, `docs/prds/application.md`, `docs/prds/workflow.md`
- Bominal ADRs: ADR-0210 §"M3 In Scope" + §"Launch-Blocking Client Bar" (closure authority), ADR-0128 superseded by ADR-0190 (corpus.lock citation contract)
- Evidence bundle template: `docs/templates/evidence-bundle-template.json`, `docs/templates/evidence-pack-template.md`
