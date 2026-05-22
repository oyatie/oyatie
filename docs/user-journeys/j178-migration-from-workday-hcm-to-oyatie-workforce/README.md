---
doc_class: User-Journey-README
journey_id: j178-migration-from-workday-hcm-to-oyatie-workforce
slice: vendor-migration-journey-wave-3-j
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Priya Menon, CHRO at Northstar Clinics, a 5K-employee health-services organization
audience_type: B2B_ENTERPRISE_CHRO
incumbent_system: Workday HCM
target_system: Oyatie workforce
source_system: workday-prod-supervisory-org
related_adrs:
  - ADR-0131-per-microservice-flat-layout
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0317-role-based-projection-unified-ux-shell
microservice_count: 15
---

# j178-migration-from-workday-hcm-to-oyatie-workforce - Workday HCM to Oyatie Workforce cutover

## At a glance

Priya Menon, CHRO at Northstar Clinics, a 5K-employee health-services organization leads a migration from Workday HCM to Oyatie workforce. The journey is not a generic persona story; it is a vendor exit path where the protagonist must preserve operational continuity while replacing named incumbent objects, APIs, permissions, reports, dashboards, and audit evidence.

- Incumbent: Workday HCM.
- Target: Oyatie workforce.
- Company: Northstar Clinics.
- Migration window: HR, payroll, benefits, and performance cutover for 5,000 employees.
- Extract mechanism: Workday EIB extract with signed integration-system-user handoff.
- Named projection: oyatie.workforce.worker_position_projection_v1.
- Parallel-run posture: two-pay-period payroll and benefits parallel run with employee self-service freeze windows.
- Stop condition: Oyatie is active, incumbent writes are frozen, rollback remains rehearsed, and all deltas are below go/no-go thresholds.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| README.md | Persona context, µservice roster, ADRs, regulatory anchors, acceptance summary | Names incumbent objects, target projection, and cutover gates |
| story.md | Full migration narrative with named milestones | Minute-by-minute migration texture, not a scaffold |
| handshake.md | Every cross-µservice and vendor-API interaction | Names caller, callee, payload, Cedar permit, audit event, and rollback |
| ux-flow.md | Migration-tool screens, progress dashboards, rollback options | Names operator controls, status states, accessibility, and failure surfaces |
| integration-test-plan.md | Verification and go/no-go plan | Parallel-run delta detection, phase gates, and rollback tests |
| schemas/cedar-policy.cedar | Authorization fragment | Principal/action/resource policy for cutover operations |
| schemas/journey-messages.proto | RPC/event contract | Migration commands, events, delta records, rollback requests |
| schemas/migration-state-machine.yaml | Lifecycle state machine | Phase transitions and terminal states |
| schemas/vendor-extract-schema.json | Source extract contract | Vendor object schema and row-hash expectations |
| schemas/cutover-runbook.json | Machine-readable cutover runbook | Hour-by-hour tasks, owners, commands, gates |

## Primary protagonist

Priya Menon, CHRO at Northstar Clinics, a 5K-employee health-services organization is accountable for the business outcome. The executive question is whether Northstar Clinics can operate on Monday, produce defensible audit evidence, and explain the decision when Workday HCM becomes read-only.

## ADR anchors

| ADR | How it constrains this migration |
|---|---|
| ADR-0131-per-microservice-flat-layout | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0145-inter-microservice-communication-reform | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0243-cedar-as-universal-gate | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0244-tenant-as-universal-scoping-primitive | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0251-compliance-pack-cell-certification-levels | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0263-observability-emission-contract | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0317-role-based-projection-unified-ux-shell | Requires tenant-scoped, Cedar-gated, auditable transitions. |

## µservice roster

| µservice | Role | Migration responsibility |
|---|---|---|
| workforce | primary | Owns employee migration state for Worker during eib-extract. |
| payroll | primary | Owns position migration state for Position during worker-position-load. |
| benefits | primary | Owns supervisory org migration state for Compensation during payroll-parallel-run. |
| compensation | primary | Owns pay group migration state for Performance during benefits-carrier-cutover. |
| performance | primary | Owns benefit election migration state for Benefit_Election during performance-retention-seal. |
| identity | supporting | Owns deduction migration state for Worker during eib-extract. |
| tenancy | supporting | Owns performance review migration state for Position during worker-position-load. |
| workflow-engine | supporting | Owns dependent migration state for Compensation during payroll-parallel-run. |
| audit-chain | supporting | Owns employee migration state for Performance during benefits-carrier-cutover. |
| compliance | supporting | Owns position migration state for Benefit_Election during performance-retention-seal. |
| drive | supporting | Owns supervisory org migration state for Worker during eib-extract. |
| messenger | supporting | Owns pay group migration state for Position during worker-position-load. |
| data-pipeline | supporting | Owns benefit election migration state for Compensation during payroll-parallel-run. |
| observability | supporting | Owns deduction migration state for Performance during benefits-carrier-cutover. |
| ops-dashboard-control-center | supporting | Owns performance review migration state for Benefit_Election during performance-retention-seal. |

## Incumbent object roster

| Incumbent object/table | Purpose | Named fields | Oyatie landing projection |
|---|---|---|---|
| Worker | Employee and contingent worker record | Worker_ID, Employee_ID, Legal_Name, Hire_Date, Worker_Type, Supervisory_Org | oyatie.workforce.worker_position_projection_v1 |
| Position | Position management object | Position_ID, Job_Profile, Supervisory_Org, Time_Type, Location, FTE | oyatie.workforce.worker_position_projection_v1 |
| Compensation | Compensation package | Compensation_Plan, Grade, Base_Pay, Allowance, Effective_Date | oyatie.workforce.worker_position_projection_v1 |
| Performance | Review and goal record | Review_ID, Period, Rating, Goal, Manager_Comment, Acknowledgement | oyatie.workforce.worker_position_projection_v1 |
| Benefit_Election | Benefit plan election | Plan_ID, Coverage_Level, Dependents, Effective_Date, Payroll_Deduction | oyatie.workforce.worker_position_projection_v1 |

## Field-mapping table

| Source field | Oyatie field | Transform rule | Evidence |
|---|---|---|---|
| Worker.Worker_ID | workforce.source_worker_id | immutable Workday worker key | audit-chain source hash and row-count proof required |
| Worker.Employee_ID | workforce.employee_number | human payroll-visible identifier | audit-chain source hash and row-count proof required |
| Position.Position_ID | workforce.position_id | pin open headcount and incumbent state | audit-chain source hash and row-count proof required |
| Position.Supervisory_Org | workforce.org_unit_id | map to Oyatie organization tree | audit-chain source hash and row-count proof required |
| Compensation.Base_Pay | compensation.base_rate | currency and frequency normalized | audit-chain source hash and row-count proof required |
| Compensation.Effective_Date | payroll.comp_effective_date | must precede first Oyatie payroll | audit-chain source hash and row-count proof required |
| Performance.Rating | performance.rating_code | region-specific visibility policy applied | audit-chain source hash and row-count proof required |
| Benefit_Election.Coverage_Level | benefits.coverage_tier | dependent eligibility verified | audit-chain source hash and row-count proof required |

## Replacement surface map

- Workday Worker Profile -> Oyatie Employee Profile.
- Workday Change Job -> Oyatie Workforce Action.
- Workday Compensation Review -> Oyatie Compensation Console.
- Workday Benefits Enrollment -> Oyatie Benefits Enrollment.
- Workday Performance Review -> Oyatie Performance Review Workspace.

## Named regulatory anchors

1. FLSA 29 CFR Part 516 payroll records retained for at least 3 years.
2. ERISA Section 107 benefit records retained for at least 6 years.
3. EEOC 29 CFR Part 1602 personnel records retained for at least 1 year and ADEA payroll records for 3 years.
4. Form I-9 retention rule: 3 years after hire or 1 year after termination, whichever is later.
5. GDPR Article 5(1)(e) storage limitation and Article 88 employment processing safeguards.
6. Korean Labor Standards Act Article 42 employee roster and wage ledger retention.

## Named milestones

- M1 EIB extract from Worker/Position/Compensation/Performance complete.
- M2 identity and supervisory-org bridge verified.
- M3 first two payroll parallel runs match.
- M4 benefits carrier files accepted.
- M5 Workday read-only archive and Oyatie workforce active.

## Acceptance summary

| AC | Required result | Evidence |
|---|---|---|
| AC-J178-001 | workforce proves Worker migration during eib-extract; FLSA 29 CFR Part 516 payroll records retained for at least 3 years remains satisfied. | EVT-J178-WORKFORCE-001 plus row-count and hash proof. |
| AC-J178-002 | payroll proves Position migration during worker-position-load; ERISA Section 107 benefit records retained for at least 6 years remains satisfied. | EVT-J178-PAYROLL-002 plus row-count and hash proof. |
| AC-J178-003 | benefits proves Compensation migration during payroll-parallel-run; EEOC 29 CFR Part 1602 personnel records retained for at least 1 year and ADEA payroll records for 3 years remains satisfied. | EVT-J178-BENEFITS-003 plus row-count and hash proof. |
| AC-J178-004 | compensation proves Performance migration during benefits-carrier-cutover; Form I-9 retention rule: 3 years after hire or 1 year after termination, whichever is later remains satisfied. | EVT-J178-COMPENSATION-004 plus row-count and hash proof. |
| AC-J178-005 | performance proves Benefit_Election migration during performance-retention-seal; GDPR Article 5(1)(e) storage limitation and Article 88 employment processing safeguards remains satisfied. | EVT-J178-PERFORMANCE-005 plus row-count and hash proof. |
| AC-J178-006 | identity proves Worker migration during eib-extract; Korean Labor Standards Act Article 42 employee roster and wage ledger retention remains satisfied. | EVT-J178-IDENTITY-006 plus row-count and hash proof. |
| AC-J178-007 | tenancy proves Position migration during worker-position-load; FLSA 29 CFR Part 516 payroll records retained for at least 3 years remains satisfied. | EVT-J178-TENANCY-007 plus row-count and hash proof. |
| AC-J178-008 | workflow-engine proves Compensation migration during payroll-parallel-run; ERISA Section 107 benefit records retained for at least 6 years remains satisfied. | EVT-J178-WORKFLOW_ENGINE-008 plus row-count and hash proof. |
| AC-J178-009 | audit-chain proves Performance migration during benefits-carrier-cutover; EEOC 29 CFR Part 1602 personnel records retained for at least 1 year and ADEA payroll records for 3 years remains satisfied. | EVT-J178-AUDIT_CHAIN-009 plus row-count and hash proof. |
| AC-J178-010 | compliance proves Benefit_Election migration during performance-retention-seal; Form I-9 retention rule: 3 years after hire or 1 year after termination, whichever is later remains satisfied. | EVT-J178-COMPLIANCE-010 plus row-count and hash proof. |
| AC-J178-011 | drive proves Worker migration during eib-extract; GDPR Article 5(1)(e) storage limitation and Article 88 employment processing safeguards remains satisfied. | EVT-J178-DRIVE-011 plus row-count and hash proof. |
| AC-J178-012 | messenger proves Position migration during worker-position-load; Korean Labor Standards Act Article 42 employee roster and wage ledger retention remains satisfied. | EVT-J178-MESSENGER-012 plus row-count and hash proof. |
| AC-J178-013 | data-pipeline proves Compensation migration during payroll-parallel-run; FLSA 29 CFR Part 516 payroll records retained for at least 3 years remains satisfied. | EVT-J178-DATA_PIPELINE-013 plus row-count and hash proof. |
| AC-J178-014 | observability proves Performance migration during benefits-carrier-cutover; ERISA Section 107 benefit records retained for at least 6 years remains satisfied. | EVT-J178-OBSERVABILITY-014 plus row-count and hash proof. |
| AC-J178-015 | ops-dashboard-control-center proves Benefit_Election migration during performance-retention-seal; EEOC 29 CFR Part 1602 personnel records retained for at least 1 year and ADEA payroll records for 3 years remains satisfied. | EVT-J178-OPS_DASHBOARD_CONTROL_CENTER-015 plus row-count and hash proof. |
| AC-J178-016 | workforce proves Worker migration during eib-extract; Form I-9 retention rule: 3 years after hire or 1 year after termination, whichever is later remains satisfied. | EVT-J178-WORKFORCE-016 plus row-count and hash proof. |
| AC-J178-017 | payroll proves Position migration during worker-position-load; GDPR Article 5(1)(e) storage limitation and Article 88 employment processing safeguards remains satisfied. | EVT-J178-PAYROLL-017 plus row-count and hash proof. |
| AC-J178-018 | benefits proves Compensation migration during payroll-parallel-run; Korean Labor Standards Act Article 42 employee roster and wage ledger retention remains satisfied. | EVT-J178-BENEFITS-018 plus row-count and hash proof. |
| AC-J178-019 | compensation proves Performance migration during benefits-carrier-cutover; FLSA 29 CFR Part 516 payroll records retained for at least 3 years remains satisfied. | EVT-J178-COMPENSATION-019 plus row-count and hash proof. |
| AC-J178-020 | performance proves Benefit_Election migration during performance-retention-seal; ERISA Section 107 benefit records retained for at least 6 years remains satisfied. | EVT-J178-PERFORMANCE-020 plus row-count and hash proof. |
| AC-J178-021 | identity proves Worker migration during eib-extract; EEOC 29 CFR Part 1602 personnel records retained for at least 1 year and ADEA payroll records for 3 years remains satisfied. | EVT-J178-IDENTITY-021 plus row-count and hash proof. |
| AC-J178-022 | tenancy proves Position migration during worker-position-load; Form I-9 retention rule: 3 years after hire or 1 year after termination, whichever is later remains satisfied. | EVT-J178-TENANCY-022 plus row-count and hash proof. |
| AC-J178-023 | workflow-engine proves Compensation migration during payroll-parallel-run; GDPR Article 5(1)(e) storage limitation and Article 88 employment processing safeguards remains satisfied. | EVT-J178-WORKFLOW_ENGINE-023 plus row-count and hash proof. |
| AC-J178-024 | audit-chain proves Performance migration during benefits-carrier-cutover; Korean Labor Standards Act Article 42 employee roster and wage ledger retention remains satisfied. | EVT-J178-AUDIT_CHAIN-024 plus row-count and hash proof. |
| AC-J178-025 | compliance proves Benefit_Election migration during performance-retention-seal; FLSA 29 CFR Part 516 payroll records retained for at least 3 years remains satisfied. | EVT-J178-COMPLIANCE-025 plus row-count and hash proof. |
| AC-J178-026 | drive proves Worker migration during eib-extract; ERISA Section 107 benefit records retained for at least 6 years remains satisfied. | EVT-J178-DRIVE-026 plus row-count and hash proof. |
| AC-J178-027 | messenger proves Position migration during worker-position-load; EEOC 29 CFR Part 1602 personnel records retained for at least 1 year and ADEA payroll records for 3 years remains satisfied. | EVT-J178-MESSENGER-027 plus row-count and hash proof. |
| AC-J178-028 | data-pipeline proves Compensation migration during payroll-parallel-run; Form I-9 retention rule: 3 years after hire or 1 year after termination, whichever is later remains satisfied. | EVT-J178-DATA_PIPELINE-028 plus row-count and hash proof. |
| AC-J178-029 | observability proves Performance migration during benefits-carrier-cutover; GDPR Article 5(1)(e) storage limitation and Article 88 employment processing safeguards remains satisfied. | EVT-J178-OBSERVABILITY-029 plus row-count and hash proof. |
| AC-J178-030 | ops-dashboard-control-center proves Benefit_Election migration during performance-retention-seal; Korean Labor Standards Act Article 42 employee roster and wage ledger retention remains satisfied. | EVT-J178-OPS_DASHBOARD_CONTROL_CENTER-030 plus row-count and hash proof. |
| AC-J178-031 | workforce proves Worker migration during eib-extract; FLSA 29 CFR Part 516 payroll records retained for at least 3 years remains satisfied. | EVT-J178-WORKFORCE-031 plus row-count and hash proof. |
| AC-J178-032 | payroll proves Position migration during worker-position-load; ERISA Section 107 benefit records retained for at least 6 years remains satisfied. | EVT-J178-PAYROLL-032 plus row-count and hash proof. |
| AC-J178-033 | benefits proves Compensation migration during payroll-parallel-run; EEOC 29 CFR Part 1602 personnel records retained for at least 1 year and ADEA payroll records for 3 years remains satisfied. | EVT-J178-BENEFITS-033 plus row-count and hash proof. |
| AC-J178-034 | compensation proves Performance migration during benefits-carrier-cutover; Form I-9 retention rule: 3 years after hire or 1 year after termination, whichever is later remains satisfied. | EVT-J178-COMPENSATION-034 plus row-count and hash proof. |
| AC-J178-035 | performance proves Benefit_Election migration during performance-retention-seal; GDPR Article 5(1)(e) storage limitation and Article 88 employment processing safeguards remains satisfied. | EVT-J178-PERFORMANCE-035 plus row-count and hash proof. |
| AC-J178-036 | identity proves Worker migration during eib-extract; Korean Labor Standards Act Article 42 employee roster and wage ledger retention remains satisfied. | EVT-J178-IDENTITY-036 plus row-count and hash proof. |

## Bespoke data packet and named failure modes

- Workforce scope: 5,000 Workers, 5,184 Positions, 5,000 Compensation packages, 4,622 active Benefit_Elections, and 3,940 Performance records.
- Priya's materiality line: one missed payroll deduction, one incorrect benefit tier, or one terminated-worker access grant blocks go-live.
- Named failure mode WD-FM-01: Workday Worker has two active Position records after a backdated transfer.
- Named failure mode WD-FM-02: Compensation.Effective_Date lands after payroll cutoff and would underpay a nurse shift differential.
- Named failure mode WD-FM-03: Benefit_Election dependent count differs from carrier eligibility file.
- Named failure mode WD-FM-04: Performance review visibility violates GDPR Article 88 manager-access scoping.
- CHRO question: "Can a nurse see tomorrow's schedule, paycheck, and benefit election without Workday?"
- Go branch: two pay-period parallel run matches gross, tax, deduction, and benefit carrier totals.
- No-go branch: Workday remains source for payroll and benefits while profile/self-service moves to Oyatie.

- Operator dialogue: Priya refuses go-live until Worker W-104882 family dental coverage is explained.
- Concrete data value: payroll gross matches at USD 9,842,118.44; benefits differ by USD 312.18.
- Evidence owner: payroll owns gross/tax parity; benefits owns carrier ACK evidence.
- Rollback owner: payroll director Jamal Reed can keep Workday payroll active for one pay period.
- Business clock: carrier ACK must arrive before the 14:00 EST payroll release.

## Deliberately out of scope

- Rewriting j01-j175 user journeys.
- Inventing a new µservice suite or hiding ownership behind a bundle.
- Taking production credentials from the incumbent system.
- Treating vendor export success as business cutover success without parallel-run deltas.
