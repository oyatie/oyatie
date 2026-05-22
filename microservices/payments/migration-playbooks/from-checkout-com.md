---
doc_class: MigrationPlaybook
from_vendor: Checkout.com
to_microservice: payments
status: draft-substance-pass
date: 2026-05-20
owner: axis-payments
related_oyatie_adrs:
  - docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
  - docs/decisions/ADR-0212-buildability-doctrine.md
  - docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
  - docs/decisions/ADR-0258-api-versioning-model.md
  - docs/decisions/ADR-0263-observability-emission-contract.md
---

# Migration Playbook: Checkout.com to Oyatie payments

## Vendor Identity + Categorization
- Vendor product family: Checkout.com global payment processing platform.
- Edition/scope: Checkout.com Core Payments with payment actions, sources, disputes, balances, processing channels, and webhooks.
- Source documentation family: Checkout.com payment APIs, reporting/export guides, webhook references, dispute tooling, vault/token docs, and settlement reports.
- Target microservice owner: axis-payments; all tenant movement remains tenant-scoped per ADR-0244.
- Classification: vendor-specific migration from Checkout.com into Oyatie payments, not a generic data-load template.
- Cell posture: run separately per tenant cell and per sovereign boundary; never pool source credentials across tenants.
- Version posture: record vendor API version, export version, schema digest, and Oyatie importer version before first extract.
- Rollback posture: the source system remains authoritative until the go/no-go gate is signed; no destructive source mutation is part of extract.

### Vendor-Specific Identity Notes
- Identity note 1: actions are the source of capture/refund chronology; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 2: processing_channel_id drives merchant routing; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 3: source tokens may represent cards, network tokens, or alternative methods; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 4: webhook type names differ from final payment status; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 5: entity_id/sub_entity_id mapping affects marketplace settlement; assess it before mapping starts because it changes target object identity or replay order.

## Pre-Migration Assessment
### Data Classes To Inventory
- Data class 1: Charges/transactions.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Charges/transactions.
  - Record failure tree for Charges/transactions: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Charges/transactions: source count remains immutable and target staging can be dropped without changing source state.
- Data class 2: Authorizations/captures/refunds.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Authorizations/captures/refunds.
  - Record failure tree for Authorizations/captures/refunds: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Authorizations/captures/refunds: source count remains immutable and target staging can be dropped without changing source state.
- Data class 3: Customers and vaulted instruments.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Customers and vaulted instruments.
  - Record failure tree for Customers and vaulted instruments: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Customers and vaulted instruments: source count remains immutable and target staging can be dropped without changing source state.
- Data class 4: Merchant accounts and processing channels.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Merchant accounts and processing channels.
  - Record failure tree for Merchant accounts and processing channels: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Merchant accounts and processing channels: source count remains immutable and target staging can be dropped without changing source state.
- Data class 5: Disputes and chargebacks.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Disputes and chargebacks.
  - Record failure tree for Disputes and chargebacks: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Disputes and chargebacks: source count remains immutable and target staging can be dropped without changing source state.
- Data class 6: Payouts/settlements/disbursements.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Payouts/settlements/disbursements.
  - Record failure tree for Payouts/settlements/disbursements: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Payouts/settlements/disbursements: source count remains immutable and target staging can be dropped without changing source state.
- Data class 7: Webhook delivery ledger.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Webhook delivery ledger.
  - Record failure tree for Webhook delivery ledger: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Webhook delivery ledger: source count remains immutable and target staging can be dropped without changing source state.
- Data class 8: Risk/fraud decisions.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Risk/fraud decisions.
  - Record failure tree for Risk/fraud decisions: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Risk/fraud decisions: source count remains immutable and target staging can be dropped without changing source state.
- Data class 9: Marketplace/split-payment records.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Marketplace/split-payment records.
  - Record failure tree for Marketplace/split-payment records: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Marketplace/split-payment records: source count remains immutable and target staging can be dropped without changing source state.
- Data class 10: 3DS/authentication evidence.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for 3DS/authentication evidence.
  - Record failure tree for 3DS/authentication evidence: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for 3DS/authentication evidence: source count remains immutable and target staging can be dropped without changing source state.
- Data class 11: Metadata/order references.
  - Inventory count, owner, last-updated timestamp, active/inactive split, attachment volume if relevant, and tenant/cell boundary for Metadata/order references.
  - Record failure tree for Metadata/order references: missing identity, missing parent, incompatible status, unsupported custom field, and policy/retention blocker.
  - Rollback observation for Metadata/order references: source count remains immutable and target staging can be dropped without changing source state.

### API Surfaces In Scope
- API surface 1: Payment/transaction API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Payment/transaction API.
  - Log observability hook `migration.extract.payments.1` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 2: Webhook event delivery.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Webhook event delivery.
  - Log observability hook `migration.extract.payments.2` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 3: Dispute/chargeback API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Dispute/chargeback API.
  - Log observability hook `migration.extract.payments.3` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 4: Settlement/reporting API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Settlement/reporting API.
  - Log observability hook `migration.extract.payments.4` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 5: Customer/vault token API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Customer/vault token API.
  - Log observability hook `migration.extract.payments.5` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.
- API surface 6: Merchant account/configuration API.
  - Capture auth method, permission scope, pagination model, rate-limit signal, retry semantics, and audit trail for Merchant account/configuration API.
  - Log observability hook `migration.extract.payments.6` with tenant, vendor, cursor, row_count, byte_count, retry_count, and checksum.

### Assessment Exit Criteria
- Schema manifest checked in to the migration evidence bundle, not source code.
- Field owners named for every custom or extension field.
- Capacity math approved for peak extract throughput, storage staging, and API backoff budget.
- Runbook contains rollback owner, communication owner, and source-system freeze owner.

## Phase 1: Extract
- Named tool: Checkout.com Payments API crawler with webhook ledger replay and report export where contracted.
- Named format: JSON payment/action/dispute pages, CSV finance reports when available, and webhook payload archive.
- Named throughput: Target 80k-220k payments/hour with action pagination; keep webhook replay to 300 events/minute until idempotency burn-in clears.
- Named rate-limits: Checkout.com API rate limit and retry headers, webhook delivery retries, endpoint pagination boundaries, and report generation availability by contract.
- Extract window: dry-run, full baseline, incremental replay, freeze delta, and final checksum pass.
- Observability: emit row-count, byte-count, cursor, job id, retry count, API remaining quota, and checksum for every chunk.
- Rollback: delete target staging partition and resume from previous source cursor; never mutate source records during extract.
### Vendor Gotchas During Extract
- Gotcha 1: actions are the source of capture/refund chronology.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `actions are the source of capture/refund chro`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 2: processing_channel_id drives merchant routing.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `processing_channel_id drives merchant routing`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 3: source tokens may represent cards, network tokens, or alternative methods.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `source tokens may represent cards, network to`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 4: webhook type names differ from final payment status.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `webhook type names differ from final payment `.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 5: entity_id/sub_entity_id mapping affects marketplace settlement.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `entity_id/sub_entity_id mapping affects marke`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.

### Extract Steps
- Step: schema snapshot.
  - Execute with the named tool and record vendor job/cursor identifiers for Checkout.com.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `schema snapshot` and resume from the prior checkpoint cursor.
- Step: baseline full extract.
  - Execute with the named tool and record vendor job/cursor identifiers for Checkout.com.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `baseline full extract` and resume from the prior checkpoint cursor.
- Step: large object partition pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Checkout.com.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `large object partition pass` and resume from the prior checkpoint cursor.
- Step: attachment or binary pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Checkout.com.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `attachment or binary pass` and resume from the prior checkpoint cursor.
- Step: incremental delta replay.
  - Execute with the named tool and record vendor job/cursor identifiers for Checkout.com.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `incremental delta replay` and resume from the prior checkpoint cursor.
- Step: freeze-window final pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Checkout.com.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `freeze-window final pass` and resume from the prior checkpoint cursor.
- Step: checksum and ledger close.
  - Execute with the named tool and record vendor job/cursor identifiers for Checkout.com.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `checksum and ledger close` and resume from the prior checkpoint cursor.

## Phase 2: Mapping
| # | Vendor object.field | Oyatie object.field | Transform | Validation |
|---:|---|---|---|---|
| 1 | `payment.id` | `payments.charge.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 2 | `payment.reference` | `payments.charge.merchant_reference` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 3 | `payment.processing_channel_id` | `payments.merchant_account.external_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 4 | `source.customer.id` | `payments.customer.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 5 | `customer.email` | `payments.customer.email` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 6 | `amount` | `payments.charge.amount_minor` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 7 | `currency` | `payments.charge.currency` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 8 | `source.type` | `payments.payment_method.type` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 9 | `source.scheme` | `payments.payment_method.brand` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 10 | `status` | `payments.charge.status` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 11 | `webhook.type` | `payments.webhook.event_type` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 12 | `approved` | `payments.authorization.approved` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 13 | `response_summary` | `payments.authorization.response_reason` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 14 | `balances.total_authorized` | `payments.charge.parent_reference` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 15 | `source.id` | `payments.payment_method.network_token_ref` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 16 | `payment_method.id` | `payments.stored_credential.external_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 17 | `reference` | `payments.order.external_reference` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 18 | `processing.acquirer_reference_number` | `payments.settlement.acquirer_reference` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 19 | `response_code` | `payments.authorization.auth_code` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 20 | `source.bin` | `payments.card.bin_prefix` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 21 | `source.last4` | `payments.card.last_four` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 22 | `risk.flagged` | `payments.risk.decision` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 23 | `marketplace.sub_entity_id` | `payments.marketplace.split_account` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 24 | `settlement.id` | `payments.settlement.batch_reference` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 25 | `dispute.id` | `payments.dispute.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 26 | `action.id` | `payments.modification.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 27 | `refund.id` | `payments.refund.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 28 | `capture.id` | `payments.capture.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 29 | `dispute.reason_code` | `payments.dispute.reason_code` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 30 | `entity_id` | `payments.legal_entity.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 31 | `source.billing_address` | `payments.billing_address.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 32 | `metadata.order_id` | `payments.metadata.order_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 33 | `metadata.invoice_id` | `payments.metadata.invoice_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 34 | `processed_on` | `payments.event.processed_at` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 35 | `requested_on` | `payments.event.requested_at` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |

### Field-Level Mapping Notes
- Mapping 1: `payment.id` becomes `payments.charge.external_vendor_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 2: `payment.reference` becomes `payments.charge.merchant_reference` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 3: `payment.processing_channel_id` becomes `payments.merchant_account.external_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 4: `source.customer.id` becomes `payments.customer.external_vendor_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 5: `customer.email` becomes `payments.customer.email` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 6: `amount` becomes `payments.charge.amount_minor` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 7: `currency` becomes `payments.charge.currency` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 8: `source.type` becomes `payments.payment_method.type` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 9: `source.scheme` becomes `payments.payment_method.brand` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 10: `status` becomes `payments.charge.status` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 11: `webhook.type` becomes `payments.webhook.event_type` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 12: `approved` becomes `payments.authorization.approved` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 13: `response_summary` becomes `payments.authorization.response_reason` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 14: `balances.total_authorized` becomes `payments.charge.parent_reference` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 15: `source.id` becomes `payments.payment_method.network_token_ref` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 16: `payment_method.id` becomes `payments.stored_credential.external_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 17: `reference` becomes `payments.order.external_reference` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 18: `processing.acquirer_reference_number` becomes `payments.settlement.acquirer_reference` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 19: `response_code` becomes `payments.authorization.auth_code` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 20: `source.bin` becomes `payments.card.bin_prefix` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 21: `source.last4` becomes `payments.card.last_four` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 22: `risk.flagged` becomes `payments.risk.decision` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 23: `marketplace.sub_entity_id` becomes `payments.marketplace.split_account` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 24: `settlement.id` becomes `payments.settlement.batch_reference` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 25: `dispute.id` becomes `payments.dispute.external_vendor_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 26: `action.id` becomes `payments.modification.external_vendor_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 27: `refund.id` becomes `payments.refund.external_vendor_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 28: `capture.id` becomes `payments.capture.external_vendor_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 29: `dispute.reason_code` becomes `payments.dispute.reason_code` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 30: `entity_id` becomes `payments.legal_entity.external_vendor_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 31: `source.billing_address` becomes `payments.billing_address.external_vendor_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 32: `metadata.order_id` becomes `payments.metadata.order_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 33: `metadata.invoice_id` becomes `payments.metadata.invoice_id` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 34: `processed_on` becomes `payments.event.processed_at` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 35: `requested_on` becomes `payments.event.requested_at` for Checkout.com.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.

## Phase 3: Cutover
- Parallel-run window: 14 calendar days with PSP still authoritative, Oyatie shadow-authorizing disabled, webhook mirror enabled, and settlement totals reconciled daily by merchant account/currency.
- Named regression-check process: payments-migration-checkout-regression-pack: auth/capture/refund/dispute/settlement replay, idempotency collision scan, and PCI redaction audit.
- Named go/no-go gate: Go when net settled amount delta is <=0.25% by currency, unsettled authorization mismatch is 0 for active captures, webhook replay loss is 0, and PCI token handling signoff is complete.
- Cutover checkpoint 1: source freeze owner confirms freeze window and active integrations are inventoried.
- Cutover checkpoint 2: target importer version, schema digest, and mapping version are frozen.
- Cutover checkpoint 3: source and target dashboards are compared from independent read paths.
- Cutover checkpoint 4: tenant communications and rollback bridge are active.
- Rollback trigger: any P0/P1 data-loss signal, unresolved regulated-data policy gap, or failed executive gate metric.
- Rollback path: restore source authority, disable target writes, replay source deltas into staging, and reopen parallel run after root cause is fixed.
### Cutover Runbook
- Cutover action 1: execute Checkout.com to Oyatie payments action lane 1 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.1` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 1 and restore the previous authority flag before retrying.
- Cutover action 2: execute Checkout.com to Oyatie payments action lane 2 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.2` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 2 and restore the previous authority flag before retrying.
- Cutover action 3: execute Checkout.com to Oyatie payments action lane 3 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.3` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 3 and restore the previous authority flag before retrying.
- Cutover action 4: execute Checkout.com to Oyatie payments action lane 4 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.4` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 4 and restore the previous authority flag before retrying.
- Cutover action 5: execute Checkout.com to Oyatie payments action lane 5 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.5` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 5 and restore the previous authority flag before retrying.
- Cutover action 6: execute Checkout.com to Oyatie payments action lane 6 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.6` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 6 and restore the previous authority flag before retrying.
- Cutover action 7: execute Checkout.com to Oyatie payments action lane 7 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.7` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 7 and restore the previous authority flag before retrying.
- Cutover action 8: execute Checkout.com to Oyatie payments action lane 8 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.8` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 8 and restore the previous authority flag before retrying.
- Cutover action 9: execute Checkout.com to Oyatie payments action lane 9 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.9` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 9 and restore the previous authority flag before retrying.
- Cutover action 10: execute Checkout.com to Oyatie payments action lane 10 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.10` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 10 and restore the previous authority flag before retrying.
- Cutover action 11: execute Checkout.com to Oyatie payments action lane 11 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.11` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 11 and restore the previous authority flag before retrying.
- Cutover action 12: execute Checkout.com to Oyatie payments action lane 12 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.12` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 12 and restore the previous authority flag before retrying.
- Cutover action 13: execute Checkout.com to Oyatie payments action lane 13 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.13` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 13 and restore the previous authority flag before retrying.
- Cutover action 14: execute Checkout.com to Oyatie payments action lane 14 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.14` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 14 and restore the previous authority flag before retrying.
- Cutover action 15: execute Checkout.com to Oyatie payments action lane 15 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.15` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 15 and restore the previous authority flag before retrying.

## Phase 4: Verification
- Named test suite: payments-migration-checkout-regression-suite.
- Named SLO targets: P95 charge lookup <100 ms, P95 refund command <180 ms, webhook replay lag <5 minutes, settlement delta <=0.25%, and duplicate idempotency writes exactly 0.
- Named delta-detection algorithm: PSP event ledger watermark plus settlement batch Merkle hash keyed by tenant/merchant/currency/reference/eventTime and late-arrival reconciliation window.
- Verification must be run from an operator account that is not the migration writer account.
- Verification evidence includes source sample, target sample, checksum, dashboard diff, policy diff, and import log digest.
- Verification check 1: cardinality parity.
  - Method: run payments-migration-checkout-regression-suite module `cardinality_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 2: field hash parity.
  - Method: run payments-migration-checkout-regression-suite module `field_hash_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 3: relationship graph parity.
  - Method: run payments-migration-checkout-regression-suite module `relationship_graph_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 4: status/lifecycle parity.
  - Method: run payments-migration-checkout-regression-suite module `status/lifecycle_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 5: owner/principal parity.
  - Method: run payments-migration-checkout-regression-suite module `owner/principal_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 6: permission/policy parity.
  - Method: run payments-migration-checkout-regression-suite module `permission/policy_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 7: attachment checksum parity.
  - Method: run payments-migration-checkout-regression-suite module `attachment_checksum_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 8: late delta replay.
  - Method: run payments-migration-checkout-regression-suite module `late_delta_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 9: dashboard/report parity.
  - Method: run payments-migration-checkout-regression-suite module `dashboard/report_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 10: audit-event continuity.
  - Method: run payments-migration-checkout-regression-suite module `audit-event_continuity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 11: SLO replay.
  - Method: run payments-migration-checkout-regression-suite module `SLO_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 12: rollback drill.
  - Method: run payments-migration-checkout-regression-suite module `rollback_drill` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.

## Phase 5: Decommission
- Named retention policy: Retain PSP raw reports for 7 years, PCI-scoped token evidence under token-retention policy, webhook payload ledger for 25 months, and settlement proofs for finance retention.
- Named teardown sequence: Disable PSP webhooks after final replay, revoke API keys, freeze merchant-account writes, archive settlement reports, remove PCI-scoped export credentials, and keep dispute portal read-only until tail risk closes.
- Decommission is not allowed until verification has two consecutive green windows and support has no open P1 migration incidents.
- Keep read-only source access long enough to satisfy support, legal hold, and finance/customer audit requirements.
- Archive migration bundle with schema manifest, mapping version, source checksum ledger, target checksum ledger, go/no-go signoff, and rollback drill result.
- Teardown step 1: disable writes for Checkout.com.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 2: archive exports for Checkout.com.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 3: revoke credentials for Checkout.com.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 4: turn off webhooks/connectors for Checkout.com.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 5: retire scheduled jobs for Checkout.com.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 6: remove temporary network access for Checkout.com.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 7: close support bridge for Checkout.com.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 8: publish final evidence for Checkout.com.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.

## Specific Failure Modes
- Failure 1: Webhook arrives before payment repair read.
  - Detection: payments-migration-checkout-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 2: Settlement report total differs from transaction ledger.
  - Detection: payments-migration-checkout-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 3: Vault token cannot be exported under PCI policy.
  - Detection: payments-migration-checkout-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 4: Dispute status changes during cutover freeze.
  - Detection: payments-migration-checkout-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 5: Merchant account currency mapping missing.
  - Detection: payments-migration-checkout-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 6: Refund references original transaction differently than charge.
  - Detection: payments-migration-checkout-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 7: Risk decision unavailable for legacy transaction.
  - Detection: payments-migration-checkout-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 8: Rate limit stalls repair pass.
  - Detection: payments-migration-checkout-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.

## Specific Tooling Estimates
| Work package | Duration | Team size | Cost band |
|---|---:|---|---:|
| PSP assessment and PCI scoping | 5-8 days | 1 payments engineer + 1 PCI lead + 1 finance SME | $28k-$55k |
| Export/replay tooling | 2-4 weeks | 3 payments/data engineers | $90k-$155k |
| Parallel run and settlement validation | 2 weeks | 2 QA + 1 finance + 1 risk SME | $45k-$90k |
| Cutover/decommission | 3-6 days | 2 engineers + release manager | $20k-$45k |

### Estimate Assumptions
- Estimate 1: PSP assessment and PCI scoping uses 1 payments engineer + 1 PCI lead + 1 finance SME for 5-8 days with $28k-$55k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 2: Export/replay tooling uses 3 payments/data engineers for 2-4 weeks with $90k-$155k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 3: Parallel run and settlement validation uses 2 QA + 1 finance + 1 risk SME for 2 weeks with $45k-$90k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.
- Estimate 4: Cutover/decommission uses 2 engineers + release manager for 3-6 days with $20k-$45k budget range.
  - Assumption: source admin access is available, export credentials are approved, and tenant-specific extensions are inventoried before build starts.
  - Risk reserve: add 20% when custom objects, sovereign cells, unusually large attachments, or regulated retention rules exceed the baseline.

## References
- https://support.checkout.com/hc/en-us/articles/25928089023378-What-is-the-API-rate-limit
- https://support.checkout.com/hc/en-us/articles/31031607845778-Recommended-payment-webhooks
- https://checkoutdocs.readme.io/docs/event-types
- docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
- docs/decisions/ADR-0212-buildability-doctrine.md
- docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
- docs/decisions/ADR-0258-api-versioning-model.md
- docs/decisions/ADR-0263-observability-emission-contract.md

## Checkpoint
- Checkpoint bundle: migration-playbooks-w2-2026-05-20 / payments / Checkout.com.
- Halt condition: playbook authored, required sections present, mapping table >=30 rows, line-count floor met, and no prior-wave playbook edited.
- Next allowed action: external review or implementation planning after VCS verify/done/promote gates pass.
