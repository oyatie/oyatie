---
doc_class: User-Journey-Handshake
journey_id: j153-devon-williams-hvac-side-business-tax-end-of-year
date: 2026-05-20
authority_tier: 2
status: draft
---

# j153 — Handshake matrix

## §1 Identity — tenant switch

`→ identity` — `POST /v1/identity/tenant-switch`

```json
{
  "principal": "devon.williams@oyamail.network",
  "from_tenant": "bayshore_climate_systems",
  "to_tenant": "devon_williams_hvac_llc",
  "intent": "year_end_reconciliation",
  "step_up_required": true
}
```

Response (`200 OK`):

```json
{
  "switch_token": "tswt_2026122819421835_8af2",
  "valid_for_seconds": 1800,
  "active_role": "sole_proprietor_admin@devon_williams_hvac_llc",
  "role_projection_view": "side_business_finops_dashboard_v1"
}
```

Cedar permit: `identity.tenant_switch`. Audit: `EVT-J153-IDENTITY-TENANT-SWITCH-OK-001`.

Failure mode: `403` with `{"reason": "step_up_failed"}` — emits `EVT-J153-IDENTITY-TENANT-SWITCH-DENY-NNN`.

## §2 Payments — reconcile multiple rails

### 2.1 Stripe Connect deposits batch approval

`→ payments` — `POST /v1/tenants/{tenant_id}/payments/reconcile/batch-approve`

Path: `tenant_id = devon_williams_hvac_llc`

```json
{
  "rail": "stripe_connect",
  "deposit_ids": ["dep_3O...", "dep_3O...", "..."],
  "deposit_count": 47,
  "expected_gross_cents": 2841927,
  "tax_year": 2026,
  "stepup_token": "tswt_..."
}
```

Response (`200 OK`):

```json
{
  "batch_id": "rcb-2026-12-28-stripe-47",
  "reconciled_count": 47,
  "discrepancies": [],
  "audit_event_id": "EVT-J153-PAYMENTS-STRIPE-RECONCILE-003"
}
```

### 2.2 Venmo Plaid-Exchange import

`→ payments` — `POST /v1/tenants/{tenant_id}/payments/imports/venmo`

```json
{
  "tax_year": 2026,
  "auth_mode": "platform_default",
  "deduplicate_against_rails": ["stripe_connect"],
  "min_threshold_cents": 0,
  "side_business_flag_required": true
}
```

Response (`202 Accepted` — async import):

```json
{
  "import_job_id": "venmo-imp-2026-12-28-3f9c",
  "expected_transaction_count": 14,
  "expected_gross_cents": 421750,
  "duplicate_count_detected": 0,
  "manual_link_required_count": 2
}
```

Audit: `EVT-J153-PAYMENTS-VENMO-IMPORT-004`.

Failure modes:

- Venmo OAuth fails → `EVT-J153-PAYMENTS-VENMO-OAUTH-FAIL-NNN`
- Duplicate detected against Stripe → emits `EVT-J153-PAYMENTS-VENMO-DUPE-DETECTED-NNN`; user must manually resolve

### 2.3 Zelle manual import

`→ payments` — `POST /v1/tenants/{tenant_id}/payments/imports/zelle/manual`

```json
{
  "tax_year": 2026,
  "transactions": [
    {"date": "2026-03-14", "customer_name": "Marisol Vargas", "amount_cents": 60000, "job_tasks_id": "J-2026-0314-MV", "rail_ref": "ZLLE-XXX"},
    {"date": "2026-07-22", "customer_name": "Marisol Vargas", "amount_cents": 60000, "job_tasks_id": "J-2026-0722-MV", "rail_ref": "ZLLE-YYY"},
    {"date": "2026-10-11", "customer_name": "Marisol Vargas", "amount_cents": 60000, "job_tasks_id": "J-2026-1011-MV", "rail_ref": "ZLLE-ZZZ"}
  ]
}
```

Response (`201`):

```json
{
  "import_id": "zelle-imp-2026-12-28-3",
  "linked_job_count": 3,
  "audit_event_id": "EVT-J153-PAYMENTS-ZELLE-MANUAL-IMPORT-005"
}
```

## §3 finops-portal — categorize + Schedule-C

### 3.1 Receipt-batch categorization (Bookie copilot)

`→ finops-portal` — `POST /v1/tenants/{tenant_id}/finops/receipts/categorize-batch`

```json
{
  "tax_year": 2026,
  "receipt_ids": ["rec_001", "rec_002", "..."],
  "receipt_count": 187,
  "copilot_id": "bookie-schedule-c-v3.2",
  "min_confidence_for_auto_apply": 0.85
}
```

Response (`200`):

```json
{
  "auto_categorized_count": 162,
  "queued_for_review_count": 25,
  "review_queue_url": "/finops/categorize/review-queue?batch=cat-2026-12-28-187"
}
```

Audit: `EVT-J153-FINOPS-RECEIPT-CAT-batch_1` through `EVT-J153-FINOPS-RECEIPT-CAT-batch_4`.

### 3.2 Per-receipt category override

`→ finops-portal` — `PUT /v1/tenants/{tenant_id}/finops/receipts/{receipt_id}/category`

```json
{
  "schedule_c_line": "EXCLUDE_NOT_BUSINESS",
  "user_override_reason": "personal_home_depot_lumber",
  "stepup_token": "tswt_..."
}
```

Response (`200`): updated record. Audit: `EVT-J153-FINOPS-RECEIPT-OVERRIDE-NNN`.

### 3.3 Year-end Schedule-C compute

`→ finops-portal` — `POST /v1/tenants/{tenant_id}/finops/year-end-reconcile`

```json
{
  "tax_year": 2026,
  "rails_included": ["stripe_connect", "venmo", "zelle"],
  "include_mileage": true,
  "include_cdtfa_draft": true,
  "produce_pdf": true,
  "produce_json": true
}
```

Response (`200`):

```json
{
  "schedule_c_id": "sc-2026-devon-williams-hvac-llc",
  "lines": {
    "line_1_gross": 3443677,
    "line_4_cogs": 714055,
    "line_7_gross_income": 2729622,
    "line_8_advertising": 61218,
    "line_9_car_truck": 14750,
    "line_10_commissions_fees": 114241,
    "line_13_depreciation": 124700,
    "line_22_supplies": 421784,
    "line_23_taxes_licenses": 108950,
    "line_27a_other": 91214,
    "line_28_total_expenses": 936857,
    "line_29_tentative_profit": 1792765,
    "line_30_home_office": 0,
    "line_31_net_profit": 1792765
  },
  "mileage": {
    "trip_count": 73,
    "miles": 4217,
    "rate_cents_per_mile": 54.5,
    "deductible_cents": 229827
  },
  "1099_k_summary": {
    "threshold_cents": 250000,
    "stripe_connect_gross_cents": 2841927,
    "venmo_gross_cents": 421750,
    "zelle_gross_cents": 180000,
    "expected_1099_k_issuers": ["stripe_connect", "venmo"]
  },
  "pdf_url": "drive://.../schedule-c-2026-devon.pdf",
  "audit_event_id": "EVT-J153-FINOPS-SCHED-C-DRAFT-CONFIRMED-009"
}
```

## §4 Tasks — mileage export

`→ tasks` — `GET /v1/tenants/{tenant_id}/tasks/mileage-log?tax_year=2026&format=irs-summary`

Response (`200`):

```json
{
  "trip_count": 73,
  "miles_total": 4217,
  "trips": [
    {
      "job_tasks_id": "J-2026-0518-AT",
      "date": "2026-05-18",
      "start_odometer": 84217,
      "end_odometer": 84263,
      "miles": 46,
      "customer_city": "San Leandro"
    }
  ],
  "irs_standard_rate_cents_per_mile_2026": 54.5,
  "deductible_total_cents": 229827
}
```

Audit: `EVT-J153-TASKS-MILEAGE-EXPORT-007`.

## §5 Compliance — CA-CDTFA filing draft

`→ compliance` — `POST /v1/tenants/{tenant_id}/regulatory-filings/cdtfa/draft`

```json
{
  "filing_year": 2026,
  "sellers_permit": "SR-FNH-12-1244419",
  "district_code": "alameda-hayward-94545",
  "base_rate_pct": 7.25,
  "district_rate_pct": 2.75,
  "combined_rate_pct": 10.25,
  "taxable_revenue_cents": 340000,
  "tax_collected_cents": 34850,
  "due_at": "2027-04-30",
  "auto_submit": false
}
```

Response (`201`):

```json
{
  "filing_id": "CDTFA-DRAFT-2026-12-28-DEVON-HVAC-LLC",
  "status": "DRAFT-AWAITING-USER-FILE",
  "due_at": "2027-04-30T23:59:59-07:00",
  "audit_event_id": "EVT-J153-COMPLIANCE-CDTFA-DRAFT-008"
}
```

## §6 Workflow-studio — publish nightly automation

`→ workflow-studio` — `POST /v1/tenants/{tenant_id}/workflow-studio/flows`

```json
{
  "name": "nightly-side-business-reconcile-v1",
  "trigger": {"type": "cron", "expr": "30 22 * * *", "timezone": "America/Los_Angeles"},
  "steps": [
    {"step": "payments.fetch_incremental", "rails": ["stripe_connect", "venmo"]},
    {"step": "finops-portal.categorize_via_bookie", "min_confidence": 0.85},
    {"step": "tasks.fetch_new_mileage_trips"},
    {"step": "finops-portal.update_schedule_c_draft"},
    {"step": "notify.push_to_owner", "summary_template": "Nightly reconcile complete: {summary}"}
  ],
  "compile_target": "workflow-engine-dag-v2"
}
```

Response (`201`):

```json
{
  "flow_id": "wfs-flow-nightly-side-biz-recon-v1-3f9c",
  "compiled_dag_id": "wfe-dag-nightly-side-biz-recon-v1-7e2a",
  "scheduled_next_run": "2026-12-29T22:30:00-08:00",
  "audit_event_id": "EVT-J153-WORKFLOW-STUDIO-NIGHTLY-PUBLISHED-010"
}
```

## §7 Connect — tax-preparer share

`→ connect` — `POST /v1/tenants/{tenant_id}/connect/tax-preparer-share`

```json
{
  "preparer_id": "cookson-tax-accounting-hayward-ca-prep-id-7741",
  "tax_year": 2026,
  "share_scope_fields": [
    "schedule_c_lines",
    "1099_k_summary",
    "mileage_total",
    "cdtfa_draft"
  ],
  "share_scope_exclusions": [
    "customer_pii",
    "bayshore_w2"
  ],
  "ttl_until": "2027-01-31T23:59:59-08:00",
  "watermark_text": "Devon Williams HVAC LLC · Cookson Tax & Accounting · 2026 Tax Year · share-id {share_id}"
}
```

Response (`201`):

```json
{
  "share_id": "sl-2026-12-28-cookson-7741",
  "share_link_url": "https://share.oya.network/tax/<opaque>",
  "expires_at": "2027-01-31T23:59:59-08:00",
  "audit_event_id": "EVT-J153-CONNECT-TAX-PREPARER-SHARE-MINT-012"
}
```

## §8 Cross-tenant forbidden imports (must produce deny)

`→ finops-portal` — `POST /v1/tenants/{tenant_id}/finops/imports/cross-tenant`

```json
{
  "source_tenant_id": "bayshore_climate_systems",
  "source_kind": "w2_payroll",
  "tax_year": 2026
}
```

Response (`403`):

```json
{
  "error": "cedar_deny",
  "reason": "adr_0311_strict_separation",
  "policy_matched": "forbid-w2-into-schedule-c",
  "audit_event_id": "EVT-J153-CEDAR-DENY-W2-INTO-SCHED-C-011"
}
```

## §9 Community — review nudge

`→ community` — `POST /v1/tenants/{tenant_id}/community/review-nudges/batch`

```json
{
  "customer_ids": ["cust_q4_001", "cust_q4_002", "..."],
  "customer_count": 12,
  "template_id": "review-request-soft-2026",
  "send_channel": "email",
  "throttle_per_customer_days": 30
}
```

Response (`202`): batch id + scheduled send time. Audit: `EVT-J153-COMMUNITY-REVIEW-NUDGE-013`.

## §10 Audit-chain seal

Per j152 §9 — same seal contract. Every event sealed with merkle proof.

## §11 Cross-µservice timing budget

| Edge | p50 | p95 | p99 |
|---|---|---|---|
| tenant switch → role projection | 110ms | 280ms | 520ms |
| Stripe Connect batch (47) → reconcile ack | 380ms | 1.1s | 2.4s |
| Venmo import job → final result | 11s | 38s | 92s |
| Receipt batch (187) categorize | 4.2s | 12s | 28s |
| Schedule-C compute → PDF | 720ms | 2.4s | 5.1s |
| workflow-studio publish → compiled DAG | 290ms | 810ms | 1.7s |
| connect share link mint | 140ms | 410ms | 880ms |
