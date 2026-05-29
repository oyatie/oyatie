# Tutorial — Hire, onboard, clock-in, and run a payroll cycle for one employee

Goal: from `Stage::OfferAccepted` to a paid first paycheck — the full first-month lifecycle for one W-2 US employee on a paid with per_seat billing_component
tenant. All on a loopback cell.

Pre-reqs:
- Loopback cell: `make dev-cell.up CELL=workplace-loopback-1 PROFILE=workplace-integration-dev`
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid with per_seat billing_component REGION=us-east-2`
- Employer payroll account pre-funded: `./bin/oya workplace payroll fund --tenant oyatie.b2b.smb.acme-software --amount-usd 100000`

## Step 1 — hire (post-offer-acceptance)

```bash
./bin/oya workplace hire \
  --tenant oyatie.b2b.smb.acme-software \
  --candidate cand-2026-05-15-aaronson \
  --legal-name "Alice Aaronson" \
  --email alice@acme-software.io \
  --employment-type W2 \
  --start-date 2026-06-01 \
  --work-state CA \
  --residence-state CA \
  --role "Senior Software Engineer" \
  --comp-base-usd 165000 \
  --pay-frequency biweekly \
  --first-payday 2026-06-12
```

Expected:
```
employee_id : emp-2026-06-01-aaronson-…
stage       : Hired
onboarding_task_count: 14
audit_event : ce-…
```

## Step 2 — kick off onboarding (3 first-day docs)

```bash
EMPLOYEE_ID=emp-2026-06-01-aaronson-…

./bin/oya workplace esign send \
  --tenant oyatie.b2b.smb.acme-software \
  --employee $EMPLOYEE_ID \
  --document offer-letter \
  --signature-level eidas-simple

./bin/oya workplace esign send \
  --tenant oyatie.b2b.smb.acme-software \
  --employee $EMPLOYEE_ID \
  --document i-9 \
  --signature-level esign-act

./bin/oya workplace esign send \
  --tenant oyatie.b2b.smb.acme-software \
  --employee $EMPLOYEE_ID \
  --document w-4-2026 \
  --signature-level esign-act
```

In dev cell Alice auto-signs within 5 s. In production she gets an email + clicks through `app.oyatie.io/sign/<sig_id>`.

Check status:
```bash
./bin/oya workplace onboarding status --tenant oyatie.b2b.smb.acme-software --employee $EMPLOYEE_ID
```
Expected: 3 docs signed, 11 remaining tasks (benefits enrollment, equipment, etc).

## Step 3 — E-Verify (federally required for US W-2)

```bash
./bin/oya workplace e-verify run \
  --tenant oyatie.b2b.smb.acme-software \
  --employee $EMPLOYEE_ID
```

Expected (dev cell):
```
status        : Employment Authorized
case_number   : 2026060100001 (dev mock)
audit_event   : ce-…
```

## Step 4 — promote to Active on start date

(`./bin/oya time advance --to 2026-06-01T08:00:00Z` on dev cell)

```bash
./bin/oya workplace employee activate \
  --tenant oyatie.b2b.smb.acme-software \
  --employee $EMPLOYEE_ID
```

Expected:
```
stage  : Active
remaining_onboarding_tasks: 0 (all complete)
```

## Step 5 — clock-in on day 1

Simulate from her device:
```bash
./bin/oya workplace clock-in \
  --tenant oyatie.b2b.smb.acme-software \
  --employee $EMPLOYEE_ID \
  --device-id "dev-iphone-mock-alice" \
  --geofence-pass true \
  --wifi-bssid "f0:9f:c2:11:23:ac" \
  --monotonic-attest "ed25519-sig-mock-A1B2C3"
```

Expected:
```
event_id : clk-2026-06-01-08-02-14-…
attestation: all-3-signals-passed
audit_event: ce-…
```

…then later, clock-out:
```bash
./bin/oya workplace clock-out --tenant oyatie.b2b.smb.acme-software --employee $EMPLOYEE_ID
```

Repeat for 10 working days; her timecard accumulates 80 hours.

## Step 6 — manager approves timecard

```bash
./bin/oya workplace timecard approve \
  --tenant oyatie.b2b.smb.acme-software \
  --employee $EMPLOYEE_ID \
  --pay-period 2026-W23 \
  --approver mgr-eve-evergreen
```

## Step 7 — run payroll cycle

```bash
./bin/oya workplace payroll run \
  --tenant oyatie.b2b.smb.acme-software \
  --pay-period 2026-W23 \
  --pay-date 2026-06-12
```

Expected (per Alice):
```
employee     : Alice Aaronson (CA W-2)
gross        : $6,346.15  ($165k/yr / 26 biweekly periods)
fed-withhold : -$762.71   (IRS Pub 15 method, single, no allowances)
ca-withhold  : -$292.81   (FTB DE-4 method B, single, 1 allowance)
ssn-employee : -$393.46   (6.2% up to $176,100 base, 2026)
ssn-employer : -$393.46   (employer match)
medicare     : -$92.02    (1.45 %)
ca-sdi       : -$57.12    (0.9 % up to $161,800 base, 2026)
net          : $4,747.97  paid to alice-checking-account
employer-tax : -$932.86   (FICA matching + FUTA + CA SUI)
audit_event  : ce-…
```

Total employer outflow: $6,346.15 gross + $932.86 employer taxes = $7,279.01.

## Step 8 — verify the paystub

```bash
./bin/oya workplace paystub show --tenant oyatie.b2b.smb.acme-software --employee $EMPLOYEE_ID --pay-period 2026-W23
```

The paystub PDF is generated, e-signed by the platform, chain-anchored, and emailed to Alice.

## Step 9 — verify quarterly filing accruals

```bash
./bin/oya workplace tax filings show --tenant oyatie.b2b.smb.acme-software --period 2026-Q2
```

Expected: form 941 (federal quarterly), DE-9 (CA quarterly) accrual entries for this payroll cycle.

## Step 10 — cleanup

```bash
make dev-tenant.delete T=oyatie.b2b.smb.acme-software
```

## What you proved

- The hire → onboarding → E-Verify → activation flow is one state machine, gated at each transition.
- E-sign with appropriate signature levels happens per document type.
- Clock-in attestation uses 3 signals; tampering breaks one and produces an audit anomaly.
- Manager approval is an explicit Cedar action distinct from clock-in.
- Payroll calculates federal + state withholding + employer taxes natively.
- Every step writes a linked audit-chain event.
- Quarterly tax filings accrue from the payroll run.
