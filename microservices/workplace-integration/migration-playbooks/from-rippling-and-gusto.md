# Migration playbook — Rippling or Gusto → Oyatie `workplace-integration`

Audience: an HRIS / People Operations engineer moving an SMB or mid-market employer from Rippling or Gusto to
`workplace-integration` without a payroll cycle gap, missed filing, or compliance break.

> Phase budget: 120 days for ≤ 500 employees; 240 days for ≤ 5,000 employees.

## Phase 0 — Inventory (Day 0…21)

1. **Employee headcount + classification:**
   ```bash
   # Rippling has a CSV export from Reports → Employees → All
   # Gusto: People → People & Pay → Employees → Export
   ```
   Save as `employees.csv`.
2. **Active state machines:**
   - Active employees with all onboarding tasks complete.
   - Pending hires not yet started.
   - Employees in offboarding (notice/separation in progress).
   - 1099 contractors.
3. **Benefits enrollments:**
   - Medical / Dental / Vision plan elections per employee.
   - 401(k) deferral percentages + employer match.
   - FSA / HSA elections.
4. **Open payroll periods:**
   - Last completed payroll cycle.
   - Pending payroll cycles.
   - Quarterly tax filings status (941, state quarterly).
5. **Documents:**
   - All employee-signed documents (offer letters, I-9, W-4, state W-equivalents, handbook ack).
   - Manager-signed documents (job descriptions, performance reviews).
6. **Time tracking:**
   - If using Rippling Time, export historical timecards.
   - If using Gusto's free time tracking, same.

## Phase 1 — Tenant + connectivity (Day 21…30)

```bash
./bin/oya tenant create \
  --id oyatie.b2b.smb.<your-org>.workplace-migration \
  --tenant-class paid with per_seat billing_component \
  --region us-east-2 \
  --pack-set "soc2-type-ii-v2017"
```

Pre-register your payroll bank account, EIN, and state tax IDs:
```bash
./bin/oya workplace tax-setup \
  --tenant oyatie.b2b.smb.<your-org>.workplace-migration \
  --fed-ein "XX-XXXXXXX" \
  --state-tax-ids "CA:XXXXXXX,NY:XXXXXXXXX,TX:XXXXXXX" \
  --payroll-bank-routing 121000358 \
  --payroll-bank-account "<mask>"
```

## Phase 2 — Employee record migration (Day 30…45)

```bash
./bin/oya workplace migrate import \
  --tenant oyatie.b2b.smb.<your-org>.workplace-migration \
  --source-format rippling \
  --source-csv employees.csv \
  --source-supplemental rippling-benefits.csv \
  --dry-run
```

(For Gusto: `--source-format gusto`.)

Review the dry-run output. Pay particular attention to:
- Date format conversions (US dates vs ISO 8601).
- State tax setup completeness.
- 1099 contractor vs W-2 classification.

Then confirm:
```bash
./bin/oya workplace migrate import ... --confirm
```

## Phase 3 — Benefits broker handoff (Day 45…60)

If keeping the same carriers (Anthem, BCBS, Aetna, Cigna, Kaiser, MetLife, Guardian, etc):
```bash
./bin/oya workplace benefits broker-link \
  --tenant oyatie.b2b.smb.<your-org>.workplace-migration \
  --carrier anthem \
  --group-id <gpid> \
  --plan-mappings ./anthem-plan-map.yaml
```

If changing brokers (e.g. moving off Rippling's broker to Oyatie partner like Justworks Health):
- Schedule open enrollment with new broker.
- Coordinate group renewal date.
- Migrate elections via Oyatie open-enrollment workflow.

## Phase 4 — Document migration + re-signing (Day 60…75)

Most documents survive in PDF; some require re-signing under Oyatie's CA:
```bash
./bin/oya workplace docs import \
  --tenant oyatie.b2b.smb.<your-org>.workplace-migration \
  --source-archive rippling-docs.zip \
  --re-sign-policy "as-is"   # keep original CA signatures
```

For I-9 specifically: keep the original under your section-3 retention.

## Phase 5 — Time tracking history (Day 75…90)

Import historical timecards for the current year (for proper YTD overtime / leave accrual):
```bash
./bin/oya workplace timecards import \
  --tenant oyatie.b2b.smb.<your-org>.workplace-migration \
  --source-format rippling \
  --source-csv timecards-2026-ytd.csv
```

## Phase 6 — Cut-over payroll cycle (Day 90…105)

Pick a payroll cycle where the old system has filed its quarterly:
1. Complete the last cycle in old system (e.g. Rippling completes 2026-W18).
2. Disable new-hire flows in old system.
3. Run reconciliation:
   ```bash
   ./bin/oya workplace migrate reconcile \
     --tenant oyatie.b2b.smb.<your-org>.workplace-migration \
     --source-format rippling \
     --through-period 2026-W18
   ```
4. First cycle on Oyatie: 2026-W19.

The reconcile step verifies YTD totals, withholding totals, employer-tax totals match between systems.

## Phase 7 — Quarterly + annual filings (Day 105…120)

For the transition quarter (e.g. 2026-Q2 if cutover was mid-quarter):
- Old system files for its portion.
- Oyatie files for its portion.
- Year-end (W-2, 1099) is a single filing from Oyatie (since the prior quarters are in Oyatie ledger).

If cutover is mid-year, you must reconcile YTD totals carefully:
```bash
./bin/oya workplace tax-reconciliation \
  --tenant oyatie.b2b.smb.<your-org>.workplace-migration \
  --source-system rippling \
  --through 2026-Q2 \
  --output reconciliation-report.json
```

## Phase 8 — Decommission old system (Day 120+)

After Q2/Q3 filings complete + 60 days clean on Oyatie:
- Cancel Rippling/Gusto subscription.
- Download all historical records (their retention obligation persists, but you should self-retain too).

## Rollback

Within the first 30 d on Oyatie:
1. Re-enable Rippling/Gusto.
2. Migrate any net-new employees back via CSV.
3. Replay any payroll cycles Oyatie ran (Rippling/Gusto support manual journal entries for this).

After Q-end filings: rollback is impractical for the quarter; you'd need to amend filings. Plan ≥ 60 d before considering rollback.

## What you gain

- One µservice replacing 2-3 vendor products.
- EU AI Act ready (for hiring algorithms).
- BLAKE3 audit chain.
- Cedar-gated workflows for fine-grained access.
- encryption-key BYOK (ADR-0251 §D-10) + sovereign deployment options.
- Lower TCO at mid-market scale.

## What you give up

- Rippling's IT-management feature breadth (SSO management, device management) — Oyatie has these via `identity` + `cloud-iac`.
- Gusto's QuickBooks Online tight integration — partner adapter exists but isn't as native.
- Vendor-specific user-base familiarity — employees will see Oyatie's UI for the first time.
