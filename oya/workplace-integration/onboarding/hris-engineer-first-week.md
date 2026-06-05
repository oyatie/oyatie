# HRIS Engineer — First Week on `workplace-integration`

Audience: an engineer with HRIS / payroll / e-sign integration experience (Workday, ADP, Gusto, Justworks, Rippling, DocuSign, Deel)
joining the `oya-workplace-integration-*` lane.

## Day 1 — required reading

- `docs/decisions/ADR-0221-workplace-integration-unified.md` — binding scope.
- US ESIGN Act (15 USC §7001-7006) — minimum federal e-sign rule.
- EU eIDAS Regulation (EU) No 910/2014 — e-sign levels (simple / advanced / qualified).
- IRS Publication 15 (Circular E) — US federal payroll withholding rules.
- FDA 21 CFR Part 11 — paid with compliance_pack gating healthcare e-sign requirements.
- KR Personal Information Protection Act — sovereign HR data residency.

Clone:
```bash
git fetch github-mirror dev
git worktree add /tmp/oyatie-lane-$USER-workplace-week1 -b onboarding/$USER-workplace-week1 github-mirror/dev
cd /tmp/oyatie-lane-$USER-workplace-week1
```

## Day 2 — walk an employee lifecycle

```bash
make dev-cell.up CELL=workplace-loopback-1 PROFILE=workplace-integration-dev
make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid with per_seat billing_component
```

Create + hire + onboard:
```bash
./bin/oya workplace employee create \
  --tenant oyatie.b2b.smb.acme-software \
  --legal-name "Alice Aaronson" \
  --email alice@acme-software.io \
  --employment-type W-2 \
  --start-date 2026-06-01 \
  --location us-ca \
  --role "Senior Software Engineer" \
  --comp-base-usd 165000

EMPLOYEE_ID=$(jq -r .id last-employee.json)

./bin/oya workplace onboarding kick-off \
  --tenant oyatie.b2b.smb.acme-software \
  --employee $EMPLOYEE_ID

./bin/oya workplace esign send \
  --tenant oyatie.b2b.smb.acme-software \
  --employee $EMPLOYEE_ID \
  --document offer-letter \
  --signature-level eidas-simple
```

The employee will (in dev cell) auto-sign within 5 s. Inspect:
```bash
./bin/oya workplace employee show --tenant oyatie.b2b.smb.acme-software --employee $EMPLOYEE_ID
```

## Day 3 — code walkthrough

1. `crates/oya-workplace-integration-domain/src/employee.rs` — `Employee` entity (joined with `ontology`'s Person).
2. `crates/oya-workplace-integration-kernel/src/lifecycle.rs` — hire → onboard → active → offboard state machine.
3. `crates/oya-workplace-integration-port-esign/src/lib.rs` — e-sign port abstraction (multiple CA backends).
4. `crates/oya-workplace-integration-port-payroll/src/lib.rs` — payroll port abstraction.
5. `crates/oya-workplace-integration-port-timeclock/src/lib.rs` — clock-in port abstraction (with attestation).
6. `crates/oya-workplace-integration-app/src/api.rs` — REST + gRPC surface.

## Day 4 — author a state-payroll-withholding rule

Pick a state from `microservices/workplace-integration/backlog/starter-state-rules.md`. Implement under
`crates/oya-workplace-integration-payroll-us-states/<state>/`:

```rust
use oya_workplace_payroll::prelude::*;

#[derive(StatePayrollRule)]
#[rule(
    state = "CA",
    effective = "2026-01-01",
    supersedes = "2025-01-01"
)]
pub struct CaliforniaPayroll2026;

impl StatePayrollRule for CaliforniaPayroll2026 {
    fn withhold(&self, ctx: &PayrollCtx) -> WithholdResult {
        // California uses Method B (Exact Calculation Method) or wage-bracket tables.
        // Simplified for tutorial: exact method.
        let annualized = ctx.pay_period_gross_usd * ctx.pay_periods_per_year as f64;
        let standard_deduction = if ctx.filing_status == FilingStatus::Single { 5_540.0 } else { 11_080.0 };
        let taxable = (annualized - standard_deduction).max(0.0);
        let tax_annual = apply_ca_brackets_2026(taxable, ctx.filing_status);
        WithholdResult::ok(tax_annual / ctx.pay_periods_per_year as f64)
    }
}
```

Hermetic tests against IRS+FTB reference scenarios are owned by Buck2 targets; run the narrow service target once it is registered, then let the trusted Rust/Prow `oya-ci-required` controller publish required evidence. Do not use retired Cargo-only loops as merge authority.

## Day 5 — ship through the GitHub adapter lane

```bash
git fetch github-mirror dev
git worktree add /tmp/oyatie-lane-workplace-add-ca-2026-payroll -b chore/workplace-add-ca-2026-payroll github-mirror/dev
cd /tmp/oyatie-lane-workplace-add-ca-2026-payroll
buck2 build //:repo-hygiene-automation-check
gh pr create --base dev --head chore/workplace-add-ca-2026-payroll --repo jason931225/oyatie
```

Open the PR through the temporary GitHub adapter. Merge readiness comes from Buck2 evidence, reviewer approval, and the trusted Rust/Prow `oya-ci-required` controller context. Compliance lanes verify the tax calculation against reference scenarios without reviving retired `oya vcs`/gate CLI authority.

## Done with week 1

- [ ] You walked a full hire → e-sign → onboard cycle end-to-end.
- [ ] You can name the 4 e-sign levels (eIDAS simple/advanced/qualified, ESIGN Act, FDA 21 CFR Part 11).
- [ ] You shipped a state-payroll rule through the GitHub adapter lane with Buck2 evidence and `oya-ci-required` green.
- [ ] You read ADR-0221 + relevant US/EU/KR regulatory references.
- [ ] You traced a clock-in event through the audit chain.

## Rookie traps

1. **Skipping signature-level matching.** Sending a CA-cleared "qualified" doc with "simple" signature level fails the regulatory
   validator. Each document type declares its minimum required level.
2. **Storing PII in audit events.** Audit events store hashed employee IDs, not names + emails.
3. **Hand-rolling payroll calculations.** Every payroll rule must derive from `StatePayrollRule` (or equivalent) + reference test cases.
4. **Ignoring multi-jurisdiction edge cases.** An employee in CA working remotely in TX has both CA and TX tax implications.
5. **Trust device clocks naively.** Use the attestation framework; ungrounded device clocks invalidate clock-in events.
