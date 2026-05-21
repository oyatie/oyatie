---
doc_class: ImplementationPlan
microservice: treasury
status: Accepted
date: 2026-05-20
owner_team: axis-treasury + axis-payments + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0252, ADR-0263, ADR-0313, ADR-0315, ADR-0319]
related_specs: [/specs/microservices/treasury.json, /specs/microservices/payments.json]
journey_id: j174-sven-eriksson-treasury-eod-position-reconciliation
ip_id: IP-016
tenant_class: paid
billing_components:
  - per_usage
sap_module_parity: TRM-CM (cash management — pooling + sweeping)
sap_trm_displacement_surface: SAP TRM Cash Management Pool Header + Pool Movements + ZBA configuration
---

# IP-016: Cash pooling with zero-balance / target-balance sweep across bank account graph

## A. Intent
Implement notional cash pools, physical cash pools, and zero-balance / target-balance account sweeps across the tenant's bank-account graph. Subsumes SAP TRM Cash Management's pool header + pool movement + ZBA configuration, and Kyriba / GTreasury cash-pooling modules. Single-PR-sized, contract-bound.

## B. Context — journey leg covered
Persona: **Sven Eriksson, Group Treasurer at NorthStream Industries (tenant: northstream-sweden, conglomerate parent)**. Sven runs EOD position reconciliation across 47 operating accounts in 14 currencies at 9 banks. Today on SAP TRM, the ZBA sweep config is a static cron + per-account flags; Sven misses Friday-to-Monday gap sweeps because the calendar in TRM doesn't know about Swedish bank holidays for the Riksbank wire window. We need: (a) holiday-aware sweep schedule, (b) per-currency target-balance rules, (c) tenant-scoped Cedar gate on sweep amounts > $5M USD-equivalent requiring CFO co-sign, and (d) audit chain proof per sweep movement.

## C. Decision
1. `cash_pool` is the root; pool members are `bank_account` rows linked via `cash_pool_member`. Pool type is closed enum: `Notional` (no movement, accounting interest only), `Physical` (real movements), `ZBA` (zero balance every cycle), `Target-Balance` (sweep to/from a target amount).
2. Sweep schedule respects the `business_calendar` of the pool master account's banking jurisdiction. Holidays come from the calendar µservice; cross-jurisdiction sweep events use the *most restrictive* calendar.
3. Sweep amount > $5M USD-equivalent (configurable per tenant; ADR-0319 middle-office gate) requires CFO co-sign via Cedar `treasury.sweep.large_amount_co_sign`. Co-sign is the recipient of `EVT-TREASURY-SWEEP-LARGE-AMOUNT-PENDING-COSIGN`.
4. Every sweep produces an immutable `sweep_movement` row plus a payment-request to payments µservice; the payment-request idempotency-key is `sweep_movement.id` to prevent double-pay on retry.

## D. Data Model Deltas
```sql
CREATE TABLE treasury.cash_pool (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  name TEXT NOT NULL,
  pool_type TEXT NOT NULL CHECK (pool_type IN ('Notional','Physical','ZBA','Target-Balance')),
  master_bank_account_id UUID NOT NULL,
  base_currency CHAR(3) NOT NULL,
  effective_from DATE NOT NULL,
  effective_to DATE,
  sweep_calendar_id UUID NOT NULL,
  large_amount_threshold_usd NUMERIC(18,2) NOT NULL DEFAULT 5000000.00,
  UNIQUE (tenant_id, name)
);

CREATE TABLE treasury.cash_pool_member (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  cash_pool_id UUID NOT NULL REFERENCES treasury.cash_pool(id),
  bank_account_id UUID NOT NULL,
  member_role TEXT NOT NULL CHECK (member_role IN ('Master','Participant','Sub-Participant')),
  target_balance NUMERIC(18,2),
  target_currency CHAR(3),
  sweep_direction TEXT CHECK (sweep_direction IN ('Up','Down','Both','Manual')),
  effective_from DATE NOT NULL,
  effective_to DATE,
  UNIQUE (cash_pool_id, bank_account_id)
);

CREATE TABLE treasury.sweep_movement (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  cash_pool_id UUID NOT NULL,
  scheduled_for TIMESTAMPTZ NOT NULL,
  from_bank_account_id UUID NOT NULL,
  to_bank_account_id UUID NOT NULL,
  amount NUMERIC(18,2) NOT NULL,
  currency CHAR(3) NOT NULL,
  amount_usd_equiv NUMERIC(18,2) NOT NULL,
  fx_rate_used NUMERIC(18,10),
  status TEXT NOT NULL CHECK (status IN ('Pending','PendingCoSign','Approved','Executed','Failed','Cancelled')),
  cedar_decision_id UUID NOT NULL,
  co_sign_principal_id UUID,
  co_signed_at TIMESTAMPTZ,
  payment_request_id UUID,
  executed_at TIMESTAMPTZ,
  failure_reason TEXT
);

CREATE INDEX ix_sweep_status_sched ON treasury.sweep_movement(tenant_id, status, scheduled_for);
```

## E. API Endpoints
```
POST   /v1/treasury/cash-pools                                         -- define pool
POST   /v1/treasury/cash-pools/{id}/members                            -- attach account
PATCH  /v1/treasury/cash-pools/{id}/members/{member_id}/target         -- update target balance
POST   /v1/treasury/cash-pools/{id}/sweep-plan
  request:  { for_date: date }
  response: 200 { movements: [...], requires_co_sign_count }

POST   /v1/treasury/sweep-movements/{id}/co-sign
  request:  { approval_comment }
  response: 200 { status: 'Approved' }

POST   /v1/treasury/sweep-movements/{id}/cancel
GET    /v1/treasury/cash-pools/{id}/position?as_of=2026-05-20T17:00Z
GET    /v1/treasury/sweep-movements?status=Executed&from=...&to=...
```

## F. Cedar Policy Hooks
```cedar
permit (
  principal in Role::"treasury-cash-manager",
  action == Action::"create_cash_pool",
  resource is Tenant
) when {
  resource.id == principal.tenant_id
};

permit (
  principal in Role::"treasury-cash-manager",
  action == Action::"execute_sweep_movement",
  resource is SweepMovement
) when {
  resource.amount_usd_equiv < resource.cash_pool.large_amount_threshold_usd ||
  (resource.co_sign_principal_id != null && resource.co_signed_at != null)
};

permit (
  principal in Role::"treasury-cfo-cosigner",
  action == Action::"co_sign_sweep_movement",
  resource is SweepMovement
) when {
  resource.tenant_id == principal.tenant_id &&
  resource.amount_usd_equiv <= principal.co_sign_ceiling_usd
};

forbid (
  principal,
  action == Action::"execute_sweep_movement",
  resource is SweepMovement
) when {
  context.now < resource.scheduled_for ||
  not_business_day(resource.scheduled_for, resource.cash_pool.sweep_calendar_id)
};
```

## G. Ontology Projection
| Vendor object | Oyatie entity | Field deltas |
|---|---|---|
| SAP TRM `BNKA` (bank master) + cash pool header | folded into `Oyatie::Treasury::CashPool` | + `large_amount_threshold_usd`, + `sweep_calendar_id` |
| SAP TRM cash pool participants table | `Oyatie::Treasury::CashPoolMember` | + `sweep_direction` (SAP infers from balance state; we make explicit) |
| SAP TRM cash position FAGL_FLEXT slice | `Oyatie::Treasury::CashPositionSnapshot` (computed) | computed read; not stored separately |
| Kyriba pool / sub-pool | folded into `Oyatie::Treasury::CashPool` recursive | |

## H. Workflow Steps
Workflow `treasury.cash_pool.daily_sweep_plan`:
1. `load_pool_state` (current balances per member from bank-statement µservice CDC)
2. `compute_required_movements` (per-member target vs actual; net direction; minimise count)
3. `fx_quote_lookup` (for cross-currency sweeps; quote source: rates µservice)
4. `classify_large_amount` (mark movements > threshold as PendingCoSign)
5. `cedar_evaluate_each`
6. `persist_movements`

Workflow `treasury.sweep.execute`:
1. `verify_status_and_window` (status=Approved AND today=scheduled_for)
2. `create_payment_request` (idempotency-key = sweep_movement.id)
3. `await_payment_callback` (with deadline = end-of-banking-day)
4. `record_execution_or_failure`
5. `emit_movement_executed` (AsyncAPI fan-out to liquidity-forecast µservice)

## I. Audit Events
- `EVT-TREASURY-CASH-POOL-CREATED` / `-MODIFIED`
- `EVT-TREASURY-SWEEP-PLAN-COMPUTED` (one per pool per day; carries movement count + total USD-equiv)
- `EVT-TREASURY-SWEEP-LARGE-AMOUNT-PENDING-COSIGN` (carries amount + co-sign target)
- `EVT-TREASURY-SWEEP-CO-SIGNED`
- `EVT-TREASURY-SWEEP-EXECUTED`
- `EVT-TREASURY-SWEEP-FAILED` (carries reason; high-signal alert)
- `EVT-TREASURY-SWEEP-CANCELLED`

## J. SLO Targets
- Sweep-plan computation for pool with ≤ 200 members p95 ≤ 1.2s.
- Co-sign request → cosigner notification p95 ≤ 2s.
- Sweep execute → payments µservice ack p95 ≤ 5s; end-to-end bank confirm SLA tracked separately.
- Position read p95 ≤ 300ms for pool with ≤ 500 movements in window.
- Idempotency replay: 100% of duplicate POSTs return the same id without creating duplicates.

Rationale: SAP TRM ZBA sweeps are nightly batch; we target intra-day re-sweeps within business window because modern treasury practice (per BIS Working Paper 1153 on intraday liquidity) needs multiple sweep windows.

## K. Failure Modes + Recovery
| Failure | Detection | Recovery |
|---|---|---|
| Payment µservice 5xx | call result | sweep_movement → `Failed`; retry policy: exponential 30s/2m/8m up to 3 retries; manual reset by treasury supervisor allowed |
| FX rate stale (> 1h old) | rate-freshness check | block sweep with `FX_RATE_STALE`; emit alarm; manual override with cedar `treasury.fx-stale-override` |
| Co-signer absent past deadline | identity presence + escalation chain | escalate to backup CFO; if none → sweep cancelled with `EVT-TREASURY-SWEEP-COSIGN-TIMEOUT` |
| Bank holiday miss (calendar µservice down) | health probe | sweep blocked with `BUSINESS_CALENDAR_UNAVAILABLE`; never default-to-execute on holiday |
| Cross-currency sweep with negative net | sanity check | reject sweep plan with structured violation; do not partial-execute |

## L. Migration Notes
Subsumes:
- SAP TRM Cash Management Pool Header + ZBA + Target Balance.
- Kyriba Cash Pooling module.
- GTreasury Liquidity module.
- Oracle Cash Management cash-pool component.

Pool-config migration: extract from SAP table `FCM_POOL_HEADER` + members; map ZBA flag to pool_type='ZBA'; recompute `large_amount_threshold_usd` per tenant policy.

## M. Cross-µservice Handoffs
- `payments`: sweep movement → payment-request (Wire / SEPA / ACH).
- `bank-statement` (substrate): CDC stream of balances feeds plan computation.
- `liquidity-forecast`: receives `SweepExecuted` to update forward cash projection.
- `audit-chain`: every plan + every movement sealed.
- `compliance`: large-amount movements feed AML transaction-monitoring per ADR-0251 finance pack.
- `cloud-finops`: cross-tenant cell-cost attribution of sweep payment fees.

## N. Acceptance criteria
- Sweep movement > threshold blocks at execute and surfaces `PendingCoSign`; co-sign by qualifying principal unblocks; non-qualifying principal yields Cedar deny + `EVT-TREASURY-SWEEP-COSIGN-DENIED`.
- Holiday on scheduled_for path: execute deferred to next business day with `EVT-TREASURY-SWEEP-HOLIDAY-DEFERRED`.
- Idempotency-replay test: same movement-id submitted twice produces one payment-request.
- Plan deterministic for the same as-of state (no random tie-breaking).
- Benchmarks named: SAP TRM Cash Management | Kyriba | GTreasury | Oracle Cash Management | Coupa Treasury (formerly Bellin).

## O. Test fixtures
- `fixtures/sweep/zba_47accounts_14ccy.json`: full Sven-scale fixture; asserts plan + co-sign routing.
- `fixtures/sweep/large_amount_cosign_path.json`: $7.2M USD-equiv movement; co-sign required.
- `fixtures/sweep/holiday_defer_riksbank.json`: scheduled Friday + Monday Riksbank holiday → asserts defer.
- `fixtures/sweep/payment_failure_retry.json`: payments 503 then 200; asserts exactly-once execute.

## P. Operational notes
The plan-compute is a constraint-satisfaction over (member.target - member.actual) minimising movement count subject to direction constraints. We use a simple greedy net-out heuristic; documented to under-optimise by ≤ 5% vs LP-solve at 1/100th the latency. Cell-local — no cross-cell sweep planning in this slice; cross-region pool members are explicitly rejected per ADR-0009 cell isolation.