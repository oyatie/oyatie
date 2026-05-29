---
doc_class: ImplementationPlan
microservice: treasury
status: Accepted
date: 2026-05-20
owner_team: axis-treasury + axis-payments + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0253, ADR-0263, ADR-0315]
related_specs: [/specs/microservices/treasury.json, /specs/microservices/payments.json, /specs/iso20022/pain-001-mapping.json]
journey_id: j106-multi-currency-cross-border-payment
ip_id: IP-017
tenant_class: paid
billing_components:
  - per_usage
sap_module_parity: TRM-TM / FIN-FSCM-IHC (in-house cash and external payment via DMEE / SWIFT) selected surfaces
sap_trm_displacement_surface: SAP DMEE format mapper + SWIFT Connector + Bank Communication Management (BCM)
---

# IP-017: Outbound payment execution via ISO 20022 pain.001.001.09 with dual-control release

## A. Intent
Implement outbound payment initiation as ISO 20022 `pain.001.001.09` Customer Credit Transfer Initiation messages, with format-per-bank profile, dual-control release, and Cedar-gated bank-channel selection. Subsumes SAP DMEE format mapping + SAP Bank Communication Management + IHC. Single-PR-sized.

## B. Context — journey leg covered
Persona: **Aïcha Diallo, Cash Operations Analyst at WAFRIA Energy (tenant: wafria-senegal)**. WAFRIA pays 200+ vendors monthly across XOF (CFA franc) domestic, EUR cross-border to French parent, and USD invoice settlements to Houston counterparties. Today, SAP DMEE produces 3 different format files (one per bank: SocGen, BNP, Citi); Aïcha emails the SocGen and BNP files via Sage Bank Connector and uploads Citi via portal manually. Two errors in 2025 caused $480k duplicated payments. We need: single canonical message (ISO 20022) + per-bank channel adapter + dual-control release + idempotency that survives bank-side retry storms.

## C. Decision
1. The internal canonical is `pain.001.001.09` (XML; serialised once; canonical-form for hashing). Per-bank adapter handles SWIFT FIN MT103 fallback, SEPA-only profile, BIC/IBAN validation, character set restriction (e.g. SWIFT character set, EBICS country-specific).
2. Dual-control: every batch of payments needs (a) preparer, (b) releaser; preparer ≠ releaser at the principal level (not just role).
3. End-to-end ID (`EndToEndId`) is the idempotency key — set to `tenant_id + ":" + sweep_or_payable_id` so bank-side de-dup is reliable; also used in our own idempotency-key check.
4. Bank channel chosen by `bank_channel_profile` row at tenant + currency + amount tier; Cedar gate validates the principal can use that channel.

## D. Data Model Deltas
```sql
CREATE TABLE treasury.payment_batch (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  batch_reference TEXT NOT NULL,
  preparer_principal_id UUID NOT NULL,
  releaser_principal_id UUID,
  prepared_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  released_at TIMESTAMPTZ,
  status TEXT NOT NULL CHECK (status IN ('Preparing','PendingRelease','Released','Submitted','PartiallyAcked','FullyAcked','Rejected','Cancelled')),
  bank_channel_profile_id UUID NOT NULL,
  iso20022_message_hash TEXT,  -- sha256 of canonical-form pain.001
  total_amount NUMERIC(18,2) NOT NULL,
  currency CHAR(3) NOT NULL,
  UNIQUE (tenant_id, batch_reference)
);

CREATE TABLE treasury.payment_instruction (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  payment_batch_id UUID NOT NULL REFERENCES treasury.payment_batch(id),
  end_to_end_id TEXT NOT NULL,  -- idempotency key for bank-side
  from_bank_account_id UUID NOT NULL,
  to_creditor_iban TEXT,
  to_creditor_bic TEXT,
  to_creditor_name TEXT NOT NULL,
  amount NUMERIC(18,2) NOT NULL,
  currency CHAR(3) NOT NULL,
  payment_purpose_code TEXT,
  remittance_info TEXT,
  status TEXT NOT NULL,
  bank_response_code TEXT,
  bank_response_at TIMESTAMPTZ,
  UNIQUE (tenant_id, end_to_end_id)
);

CREATE TABLE treasury.bank_channel_profile (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  bank_id UUID NOT NULL,
  channel TEXT NOT NULL CHECK (channel IN ('SWIFT-FIN','SWIFT-SWIFTNet-FileAct','EBICS-T','EBICS-TS','API-Direct','SFTP-PainXML','Host2Host')),
  format_variant TEXT NOT NULL,  -- e.g. 'pain.001.001.09', 'MT103-STP', 'CGI-XML-CH-Variant'
  supported_currencies CHAR(3)[] NOT NULL,
  amount_tier_max NUMERIC(18,2),
  cutoff_time TIME NOT NULL,
  cutoff_timezone TEXT NOT NULL
);
```

## E. API Endpoints
```
POST   /v1/treasury/payment-batches
  request:  { batch_reference, instructions: [...], bank_channel_profile_id }
  response: 201 { batch_id, status: 'Preparing' }

POST   /v1/treasury/payment-batches/{id}/prepare
  -- triggers ISO 20022 serialise + hash + per-instruction validation
  response: 200 { iso20022_message_hash, validation_findings: [...] }

POST   /v1/treasury/payment-batches/{id}/release
  request:  { release_comment }
  response: 200 { status: 'Released', released_at }
  errors:   403 SAME_PRINCIPAL_DUAL_CONTROL_VIOLATION, 412 HASH_MISMATCH

POST   /v1/treasury/payment-batches/{id}/cancel
GET    /v1/treasury/payment-batches/{id}                          -- includes per-instruction status + bank acks
GET    /v1/treasury/payment-batches/{id}/iso20022.xml             -- canonical XML render
```

## F. Cedar Policy Hooks
```cedar
permit (
  principal in Role::"treasury-payment-preparer",
  action == Action::"prepare_payment_batch",
  resource is Tenant
) when {
  resource.id == principal.tenant_id
};

permit (
  principal in Role::"treasury-payment-releaser",
  action == Action::"release_payment_batch",
  resource is PaymentBatch
) when {
  resource.tenant_id == principal.tenant_id &&
  resource.preparer_principal_id != principal.id &&  // dual-control
  resource.total_amount <= principal.release_ceiling
};

permit (
  principal in Role::"treasury-channel-admin",
  action == Action::"use_bank_channel",
  resource is BankChannelProfile
) when {
  resource.tenant_id == principal.tenant_id
};

forbid (
  principal,
  action == Action::"release_payment_batch",
  resource is PaymentBatch
) when {
  context.now > resource.bank_channel_profile.cutoff_at_today_local  // past cutoff → next day
};
```

## G. Ontology Projection
| Vendor object | Oyatie entity | Field deltas |
|---|---|---|
| SAP `REGUH` (payment header) | folded into `Oyatie::Treasury::PaymentBatch` | + `iso20022_message_hash`, + dual-control fields |
| SAP `REGUP` (payment items) | `Oyatie::Treasury::PaymentInstruction` | + `end_to_end_id` (idempotency), normalized IBAN/BIC fields |
| SAP DMEE configuration | folded into `Oyatie::Treasury::BankChannelProfile.format_variant` | DMEE flexibility is captured per-bank-profile, not per-payment |
| SAP BCM (Bank Communication Management) | folded into `Oyatie::Treasury::BankChannelProfile.channel` | |

## H. Workflow Steps
Workflow `treasury.payment.prepare`:
1. `validate_instructions` (IBAN/BIC check via rates µservice or SWIFT BIC dir cache; mandatory-fields gate; currency match vs from-account)
2. `apply_bank_profile_restrictions` (e.g. SocGen rejects non-ASCII remittance info; we transliterate per profile rule and emit findings)
3. `serialize_iso20022_canonical` (XML; xml-c14n; UTF-8; LF line endings)
4. `hash_message` (sha256)
5. `mark_pending_release`

Workflow `treasury.payment.release`:
1. `cedar_dual_control_check`
2. `compare_hash_with_prepared` (412 if mismatch)
3. `mark_released`
4. `submit_to_bank_channel` (calls payments µservice adapter)
5. `emit_payment_batch_released`

Workflow `treasury.payment.bank_ack_ingest`:
- Per-instruction ack callback updates `bank_response_code` + status; batch status rolls up to PartiallyAcked / FullyAcked.

## I. Audit Events
- `EVT-TREASURY-PAYMENT-BATCH-PREPARED` (with hash)
- `EVT-TREASURY-PAYMENT-BATCH-RELEASED` (with releaser + cedar_decision_id)
- `EVT-TREASURY-PAYMENT-BATCH-HASH-MISMATCH-REJECTED`
- `EVT-TREASURY-PAYMENT-BATCH-CUTOFF-MISSED`
- `EVT-TREASURY-PAYMENT-INSTRUCTION-ACKED` / `-NACKED`
- `EVT-TREASURY-PAYMENT-DUAL-CONTROL-VIOLATION-ATTEMPTED` (always; security signal)

## J. SLO Targets
- Batch prepare with 500 instructions p95 ≤ 1.8s (including IBAN validation).
- Release → bank submit p95 ≤ 800ms.
- Bank ack ingest p95 ≤ 200ms; end-to-end submit → first ack tracked as SLA per bank.
- ISO 20022 message size cap: 32 MB per batch (split otherwise); 5000 instructions per batch max.
- Idempotency: 100% of duplicate releases (same hash) produce identical submission with same end_to_end_ids.

## K. Failure Modes + Recovery
| Failure | Detection | Recovery |
|---|---|---|
| Instruction validation fail (bad IBAN check digit) | mod-97 check | instruction marked `Invalid`; batch cannot release until removed/fixed |
| Hash mismatch at release (someone edited after prepare) | sha256 compare | 412 + `EVT-TREASURY-PAYMENT-BATCH-HASH-MISMATCH-REJECTED`; re-prepare required |
| Cutoff missed | Cedar forbid | 410 GONE; batch must be rescheduled or cancelled |
| Bank channel down | health probe + connect timeout | submit queued; retry policy per profile; release status unchanged until success or manual cancel |
| Bank NACK on some instructions | callback parsing | per-instruction NACKED; batch becomes PartiallyAcked; failure_reason carries bank reason code |

## L. Migration Notes
Subsumes:
- SAP TRM/TR-TM payment execution + SAP DMEE format generator + SAP BCM channel.
- SAP S/4 In-House Cash (IHC).
- Kyriba payment factory.
- Coupa Treasury payment hub.

Canonical mapping: every legacy payment-format-specific transform is replaced by a single canonical pain.001 + per-bank profile that can degrade to MT103 if a bank doesn't accept ISO 20022 (still mandatory until 2027 per SWIFT CBPR+ deadline).

## M. Cross-µservice Handoffs
- `payments` (substrate): channel adapters (SWIFT, EBICS, API-Direct).
- `bank-statement`: matches outbound payment to ack via end_to_end_id.
- `accounts-payable`: source of payable that triggers the instruction.
- `audit-chain`: full ISO 20022 XML retained for 10y per financial-records-retention pack.
- `compliance`: sanctions screening on creditor name + BIC before release (synchronous gate); ADR-0251 finance pack.
- `cash-position` (within treasury): updated on each ack.

## N. Acceptance criteria
- Preparer cannot release; releaser cannot prepare same batch (Cedar deny on attempt).
- Editing batch post-prepare produces hash mismatch + clear error.
- Sanctions hit on a single instruction blocks the *whole* batch from release with structured violation list.
- ISO 20022 XML validates against XSD `pain.001.001.09.xsd`; sample fixtures included.
- Benchmarks named: SAP TRM Payment Hub | Kyriba Payments | Coupa Treasury Payments | Oracle Cash Management Payments | GTreasury Payments.

## O. Test fixtures
- `fixtures/payments/200_instructions_3currencies.xml`: full batch passes XSD validation.
- `fixtures/payments/dual_control_violation_attempt.json`: same principal tries both; asserts 403.
- `fixtures/payments/cutoff_miss_5pm.json`: scheduled 5:02pm with 5:00pm cutoff → 410.
- `fixtures/payments/iban_checkdigit_invalid.json`: instruction blocked at prepare with explicit reason.
- `fixtures/payments/sanctions_creditor_hit.json`: sanctions µservice returns hit; batch release denied.

## P. Operational notes
We persist the canonical XML (gzipped) for 10y; hash is the immutable cross-reference. Per-bank profile cutoff handling uses banking-day calendar from calendar µservice; profile.cutoff_time + tz drives the `cutoff_at_today_local`. Sanctions screening is in-line and blocks; we never settle then-rollback.