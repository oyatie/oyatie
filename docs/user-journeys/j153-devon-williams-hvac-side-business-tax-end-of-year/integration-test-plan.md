---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j153-devon-williams-hvac-side-business-tax-end-of-year
date: 2026-05-20
authority_tier: 2
status: draft
---

# j153 — Integration test plan

## Test environment

| Component | Source |
|---|---|
| Seed tenants | `tests/fixtures/tenants/{bayshore_climate_systems,devon_williams_hvac_llc}.yaml` |
| Seed user | `tests/fixtures/personas/devon-williams.yaml` (dual-tenant) |
| Seed Stripe deposits | `tests/fixtures/payments/stripe-connect-2026-devon-47-deposits.jsonl` |
| Seed Venmo transactions | `tests/fixtures/payments/venmo-2026-devon-14-transactions.json` |
| Seed Zelle entries | `tests/fixtures/payments/zelle-2026-marisol-3.json` |
| Seed receipts | `tests/fixtures/receipts/devon-2026-187-receipts/*` (OCR'd) |
| Seed mileage trips | `tests/fixtures/tasks/devon-2026-73-trips.csv` |
| Mock Cookson Tax | `tests/mocks/cookson-tax-recipient.toml` |
| Mock Stripe | `tests/mocks/stripe-connect.toml` (returns 1099-K when threshold crossed) |
| Mock Venmo OAuth | `tests/mocks/venmo-oauth.toml` |
| Mock CDTFA submission endpoint | `tests/mocks/cdtfa-submission.toml` |
| Frozen clock | `2026-12-28T19:42:00-08:00` |

## Tests

### T-J153-001 — Tenant switch + role projection

**Action:** POST `/v1/identity/tenant-switch` with valid attestation.

**Pass:** switch_token issued; active_role = `sole_proprietor_admin@devon_williams_hvac_llc`; role_projection_view = `side_business_finops_dashboard_v1`. Audit `EVT-J153-IDENTITY-TENANT-SWITCH-OK-001` sealed.

**Fail:** Any field of the response indicates the Bayshore tenant role.

### T-J153-002 — Stripe batch reconcile

**Action:** POST `/v1/tenants/devon_williams_hvac_llc/payments/reconcile/batch-approve` with 47 deposit_ids.

**Pass:** 47/47 reconciled; expected_gross_cents = 2841927 exactly; no discrepancies. Audit `EVT-J153-PAYMENTS-STRIPE-RECONCILE-003`.

**Fail:** Any discrepancy in dollar amount; any deposit_id missing.

### T-J153-003 — Venmo import with deduplication

**Action:** POST `/v1/tenants/.../payments/imports/venmo`. Mock returns 14 transactions, 2 of which intentionally overlap with Stripe deposits (same dollar, same date, same customer name).

**Pass:** Final imported count = 12, not 14. The 2 overlapping flagged as `EVT-J153-PAYMENTS-VENMO-DUPE-DETECTED-NNN`. User confirmation required before final import.

**Fail:** Overlaps imported as new transactions (double-counts income).

### T-J153-004 — Zelle manual import

**Action:** POST `/v1/tenants/.../payments/imports/zelle/manual` with 3 entries.

**Pass:** All 3 linked to existing tasks job IDs. linked_job_count = 3. Audit `EVT-J153-PAYMENTS-ZELLE-MANUAL-IMPORT-005`.

**Fail:** Any entry created without a job link, or any duplicate against Stripe/Venmo.

### T-J153-005 — 1099-K threshold computation

**Action:** POST `/v1/tenants/.../finops/threshold-compute` (internal trigger after T-J153-002 + T-J153-003 + T-J153-004).

**Pass:** Threshold = $2,500. Stripe gross $28,419.27 (above). Venmo gross $4,217.50 (above). Zelle $1,800 (below, but Zelle doesn't issue). expected_1099_k_issuers = ["stripe_connect", "venmo"].

**Fail:** Zelle marked as 1099-K issuer; threshold computed wrongly.

### T-J153-006 — Receipt batch categorization

**Action:** POST `/v1/tenants/.../finops/receipts/categorize-batch` with 187 receipt_ids.

**Pass:** auto_categorized_count ≥ 160 (most should auto); queued_for_review_count ≤ 27; review queue populated correctly. Audit `EVT-J153-FINOPS-RECEIPT-CAT-batch_*`.

**Fail:** Confidence threshold violated (a receipt auto-applied below 0.85); or a receipt categorized to a non-existent Schedule-C line.

### T-J153-007 — Personal-trip exclusion (override)

**Action:** PUT a single receipt (the Home Depot lumber) to `EXCLUDE_NOT_BUSINESS`.

**Pass:** Receipt excluded from Schedule-C lines; Bookie's confidence model updated. Audit `EVT-J153-FINOPS-RECEIPT-OVERRIDE-NNN`.

**Fail:** Receipt still appears in any Schedule-C line.

### T-J153-008 — Schedule-C compute with mileage on Form 4562

**Action:** POST `/v1/tenants/.../finops/year-end-reconcile`.

**Pass:** lines.line_1_gross = 3443677. lines.line_31_net_profit = 1792765. mileage.deductible_cents = 229827 (4,217 × 54.5). Mileage shown on Form 4562 path, NOT subtracted from line 9. PDF produced.

**Fail:** Any line value diverges by more than $0.01; mileage double-counted; PDF missing.

### T-J153-009 — CDTFA filing draft

**Action:** POST `/v1/tenants/.../regulatory-filings/cdtfa/draft`.

**Pass:** filing_id created; tax_collected_cents = 34850 (correct on 10.25% × $3,400 = $348.50); due_at = 2027-04-30; auto_submit = false. Audit `EVT-J153-COMPLIANCE-CDTFA-DRAFT-008`.

**Fail:** Auto-submitted (filing should be queued, not filed). Tax rate computed wrongly.

### T-J153-010 — workflow-studio nightly publish

**Action:** POST `/v1/tenants/.../workflow-studio/flows` with the cron `30 22 * * * America/Los_Angeles`.

**Pass:** flow_id + compiled_dag_id returned; scheduled_next_run = 2026-12-29T22:30:00-08:00. workflow-engine accepts the compiled DAG without warnings.

**Fail:** DAG fails to compile; trigger time off by >60s.

### T-J153-011 — Cross-tenant W-2 import refused (Cedar deny)

**Action:** POST `/v1/tenants/.../finops/imports/cross-tenant` with `source_tenant_id = bayshore_climate_systems`, `source_kind = w2_payroll`.

**Pass:** 403 returned with reason `adr_0311_strict_separation`. Audit `EVT-J153-CEDAR-DENY-W2-INTO-SCHED-C-011`. No data imported.

**Fail:** 200 returned; any data imported; cross-tenant boundary crossed.

### T-J153-012 — Tax-preparer share mint

**Action:** POST `/v1/tenants/.../connect/tax-preparer-share` with valid scope.

**Pass:** share_link_url issued; ttl = Jan 31 2027 23:59 PST; PDF includes watermark text with share_id; customer_pii is NOT included. Audit `EVT-J153-CONNECT-TAX-PREPARER-SHARE-MINT-012`.

**Fail:** Share includes Bayshore W-2 (boundary leak); customer_pii leaks; missing watermark.

### T-J153-013 — Preparer-side download

**Action:** Mock Cookson clicks the share-link.

**Pass:** Each download emits `EVT-J153-CONNECT-PREPARER-DOWNLOAD-NNN` with the preparer identity, share_id, and download timestamp.

**Fail:** Download not sealed; preparer identity missing.

### T-J153-014 — Share-link expiry

**Action:** Advance clock to 2027-02-01T00:00. Mock Cookson attempts download.

**Pass:** 410 Gone returned; audit `EVT-J153-CONNECT-SHARE-EXPIRED-NNN`. The downloads already taken before expiry are still valid (already on Cookson's disk).

**Fail:** Download succeeds after TTL.

### T-J153-015 — Community review nudge throttle

**Action:** POST review nudges to the 12 customers. Then immediately attempt POST again (same customers).

**Pass:** Second batch is rejected with throttle violation; only the first send goes out. Audit emits a throttle-skip event.

**Fail:** Two nudges sent to the same customer within 30 days.

### T-J153-016 — Nightly automation first run

**Action:** Advance clock to 2026-12-29T22:30:00. Add 2 new Stripe deposits + 1 new mileage trip into fixtures.

**Pass:** Flow fires; both deposits auto-categorized (Bookie confidence ≥0.85 on repeat customers); 1 trip added; Schedule-C draft updated. Push notification fires to Devon. Audit `EVT-J153-WORKFLOW-NIGHTLY-RECONCILE-RUN-014`.

**Fail:** Flow doesn't fire; categorization fails silently; push notification missing.

### T-J153-017 — Tenant scoping invariant

**Action:** Query each µservice's audit log for all journey j153 events.

**Pass:** Every event carries `tenant_id = devon_williams_hvac_llc`. Bayshore tenant ID NEVER appears in side-business events.

**Fail:** Any event missing or wrong tenant_id.

### T-J153-018 — IRS amounts vs. seeded source-of-truth

**Action:** Manual cross-check of computed Schedule-C lines against a hand-computed expected file `tests/fixtures/expected/schedule-c-2026-devon-hand-computed.json`.

**Pass:** All 11 line values match within $0.01.

**Fail:** Any line value diverges.

### T-J153-019 — Soak: 30-day nightly automation

**Action:** Simulated 30 nightly runs with random small transaction injections.

**Pass:** ≥27 runs (90%) complete within 12 minutes; each emits the right audit event class; Bookie's auto-categorize rate ≥80% across the run.

**Fail:** <90% completion rate; Bookie's auto-categorize rate drops below 80%.

### T-J153-020 — Audit-chain merkle integrity

**Action:** Export all journey j153 events; verify merkle proofs.

**Pass:** All events validate against the day's epoch root; chain has no gaps.

**Fail:** Any event missing or any proof invalid.

## AC mapping

| Test | Maps to AC |
|---|---|
| T-J153-001 | AC-J153-009 |
| T-J153-002 | AC-J153-001 |
| T-J153-003 | AC-J153-002 |
| T-J153-004 | AC-J153-003 |
| T-J153-005 | AC-J153-002 |
| T-J153-006 | AC-J153-004 |
| T-J153-007 | AC-J153-004 |
| T-J153-008 | AC-J153-005 + ALL |
| T-J153-009 | AC-J153-006 |
| T-J153-010 | AC-J153-007 |
| T-J153-011 | AC-J153-009 |
| T-J153-012 | AC-J153-008 |
| T-J153-013 | AC-J153-008 |
| T-J153-014 | AC-J153-008 |
| T-J153-015 | Community guardrail |
| T-J153-016 | AC-J153-007 |
| T-J153-017 | ALL |
| T-J153-018 | AC-J153-005 |
| T-J153-019 | AC-J153-007 |
| T-J153-020 | AC-J153-010 |
