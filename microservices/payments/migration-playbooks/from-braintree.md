---
doc_class: MigrationPlaybook
from_vendor: Braintree
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

# Migration Playbook: Braintree to Oyatie payments

## Vendor Identity + Categorization
- Vendor product family: PayPal Braintree full-stack payment gateway.
- Edition/scope: Braintree gateway with merchant accounts, transactions, customers, vaulted payment methods, disputes, webhooks, and disbursements.
- Source documentation family: Braintree payment APIs, reporting/export guides, webhook references, dispute tooling, vault/token docs, and settlement reports.
- Target microservice owner: axis-payments; all tenant movement remains tenant-scoped per ADR-0244.
- Classification: vendor-specific migration from Braintree into Oyatie payments, not a generic data-load template.
- Cell posture: run separately per tenant cell and per sovereign boundary; never pool source credentials across tenants.
- Version posture: record vendor API version, export version, schema digest, and Oyatie importer version before first extract.
- Rollback posture: the source system remains authoritative until the go/no-go gate is signed; no destructive source mutation is part of extract.

### Vendor-Specific Identity Notes
- Identity note 1: merchantAccountId controls currency and settlement; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 2: transaction status lifecycle includes authorized/submitted/settling/settled; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 3: vault tokens may not be exportable without PCI workflow; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 4: disbursement dates trail transaction dates; assess it before mapping starts because it changes target object identity or replay order.
- Identity note 5: disputes may be linked by transaction id rather than PSP reference; assess it before mapping starts because it changes target object identity or replay order.

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
- Named tool: Braintree GraphQL/REST transaction search plus Control Panel export and webhook replay harness.
- Named format: CSV transaction/customer exports, JSON GraphQL repair reads, webhook notification archive, and vault token manifest.
- Named throughput: Target 60k-180k transactions/hour; Braintree search caps require partitioned createdAt windows under result limits.
- Named rate-limits: Braintree search result limits, API rate behavior, webhook retry cadence, settlement batch availability, and PayPal account permission boundaries.
- Extract window: dry-run, full baseline, incremental replay, freeze delta, and final checksum pass.
- Observability: emit row-count, byte-count, cursor, job id, retry count, API remaining quota, and checksum for every chunk.
- Rollback: delete target staging partition and resume from previous source cursor; never mutate source records during extract.
### Vendor Gotchas During Extract
- Gotcha 1: merchantAccountId controls currency and settlement.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `merchantAccountId controls currency and settl`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 2: transaction status lifecycle includes authorized/submitted/settling/settled.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `transaction status lifecycle includes authori`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 3: vault tokens may not be exportable without PCI workflow.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `vault tokens may not be exportable without PC`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 4: disbursement dates trail transaction dates.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `disbursement dates trail transaction dates`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.
- Gotcha 5: disputes may be linked by transaction id rather than PSP reference.
  - Preventive control: add preflight probe and a named quarantine bucket for records affected by `disputes may be linked by transaction id rath`.
  - Recovery action: pause only the affected object family, keep other families extracting, and re-run the cursor after the control is fixed.
  - Evidence: attach sample source id, target staging id, checksum, and operator note to the checkpoint bundle.

### Extract Steps
- Step: schema snapshot.
  - Execute with the named tool and record vendor job/cursor identifiers for Braintree.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `schema snapshot` and resume from the prior checkpoint cursor.
- Step: baseline full extract.
  - Execute with the named tool and record vendor job/cursor identifiers for Braintree.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `baseline full extract` and resume from the prior checkpoint cursor.
- Step: large object partition pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Braintree.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `large object partition pass` and resume from the prior checkpoint cursor.
- Step: attachment or binary pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Braintree.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `attachment or binary pass` and resume from the prior checkpoint cursor.
- Step: incremental delta replay.
  - Execute with the named tool and record vendor job/cursor identifiers for Braintree.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `incremental delta replay` and resume from the prior checkpoint cursor.
- Step: freeze-window final pass.
  - Execute with the named tool and record vendor job/cursor identifiers for Braintree.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `freeze-window final pass` and resume from the prior checkpoint cursor.
- Step: checksum and ledger close.
  - Execute with the named tool and record vendor job/cursor identifiers for Braintree.
  - Expected state delta: staging row count increases, source remains unchanged, and checkpoint cursor advances monotonically.
  - Rollback: drop the target staging batch for `checksum and ledger close` and resume from the prior checkpoint cursor.

## Phase 2: Mapping
| # | Vendor object.field | Oyatie object.field | Transform | Validation |
|---:|---|---|---|---|
| 1 | `transaction.id` | `payments.charge.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 2 | `transaction.orderId` | `payments.charge.merchant_reference` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 3 | `merchantAccountId` | `payments.merchant_account.external_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 4 | `customer.id` | `payments.customer.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 5 | `customer.email` | `payments.customer.email` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 6 | `amount` | `payments.charge.amount_minor` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 7 | `currencyIsoCode` | `payments.charge.currency` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 8 | `paymentInstrumentType` | `payments.payment_method.type` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 9 | `creditCard.cardType` | `payments.payment_method.brand` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 10 | `status` | `payments.charge.status` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 11 | `disbursement.type` | `payments.webhook.event_type` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 12 | `processorResponseCode` | `payments.authorization.approved` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 13 | `processorResponseText` | `payments.authorization.response_reason` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 14 | `refundedTransactionId` | `payments.charge.parent_reference` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 15 | `creditCard.token` | `payments.payment_method.network_token_ref` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 16 | `paymentMethodToken` | `payments.stored_credential.external_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 17 | `customFields.orderId` | `payments.order.external_reference` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 18 | `processorSettlementResponseCode` | `payments.settlement.acquirer_reference` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 19 | `processorAuthorizationCode` | `payments.authorization.auth_code` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 20 | `creditCard.bin` | `payments.card.bin_prefix` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 21 | `creditCard.last4` | `payments.card.last_four` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 22 | `riskData.decision` | `payments.risk.decision` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 23 | `serviceFeeAmount` | `payments.marketplace.split_account` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 24 | `disbursement.id` | `payments.settlement.batch_reference` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 25 | `dispute.id` | `payments.dispute.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 26 | `refund.id` | `payments.modification.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 27 | `refundIds` | `payments.refund.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 28 | `partialSettlementTransactionIds` | `payments.capture.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 29 | `dispute.reason` | `payments.dispute.reason_code` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 30 | `subMerchantAccountId` | `payments.legal_entity.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 31 | `billingAddress.id` | `payments.billing_address.external_vendor_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 32 | `customFields.invoiceId` | `payments.metadata.order_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 33 | `customFields.cartId` | `payments.metadata.invoice_id` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 34 | `createdAt` | `payments.event.processed_at` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |
| 35 | `updatedAt` | `payments.event.requested_at` | map with PSP-specific normalization and immutable idempotency key | round-trip value matches PSP ledger or is explicitly quarantined |

### Field-Level Mapping Notes
- Mapping 1: `transaction.id` becomes `payments.charge.external_vendor_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 2: `transaction.orderId` becomes `payments.charge.merchant_reference` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 3: `merchantAccountId` becomes `payments.merchant_account.external_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 4: `customer.id` becomes `payments.customer.external_vendor_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 5: `customer.email` becomes `payments.customer.email` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 6: `amount` becomes `payments.charge.amount_minor` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 7: `currencyIsoCode` becomes `payments.charge.currency` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 8: `paymentInstrumentType` becomes `payments.payment_method.type` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 9: `creditCard.cardType` becomes `payments.payment_method.brand` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 10: `status` becomes `payments.charge.status` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 11: `disbursement.type` becomes `payments.webhook.event_type` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 12: `processorResponseCode` becomes `payments.authorization.approved` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 13: `processorResponseText` becomes `payments.authorization.response_reason` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 14: `refundedTransactionId` becomes `payments.charge.parent_reference` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 15: `creditCard.token` becomes `payments.payment_method.network_token_ref` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 16: `paymentMethodToken` becomes `payments.stored_credential.external_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 17: `customFields.orderId` becomes `payments.order.external_reference` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 18: `processorSettlementResponseCode` becomes `payments.settlement.acquirer_reference` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 19: `processorAuthorizationCode` becomes `payments.authorization.auth_code` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 20: `creditCard.bin` becomes `payments.card.bin_prefix` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 21: `creditCard.last4` becomes `payments.card.last_four` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 22: `riskData.decision` becomes `payments.risk.decision` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 23: `serviceFeeAmount` becomes `payments.marketplace.split_account` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 24: `disbursement.id` becomes `payments.settlement.batch_reference` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 25: `dispute.id` becomes `payments.dispute.external_vendor_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 26: `refund.id` becomes `payments.modification.external_vendor_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 27: `refundIds` becomes `payments.refund.external_vendor_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 28: `partialSettlementTransactionIds` becomes `payments.capture.external_vendor_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 29: `dispute.reason` becomes `payments.dispute.reason_code` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 30: `subMerchantAccountId` becomes `payments.legal_entity.external_vendor_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 31: `billingAddress.id` becomes `payments.billing_address.external_vendor_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 32: `customFields.invoiceId` becomes `payments.metadata.order_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 33: `customFields.cartId` becomes `payments.metadata.invoice_id` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 34: `createdAt` becomes `payments.event.processed_at` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.
- Mapping 35: `updatedAt` becomes `payments.event.requested_at` for Braintree.
  - Transform detail: map with PSP-specific normalization and immutable idempotency key; record source raw value and normalized target value in the evidence ledger.
  - Validation detail: round-trip value matches PSP ledger or is explicitly quarantined; include one paiden sample and one edge sample for this field when it is active in the tenant.
  - Rollback detail: remove target staging rows keyed by the vendor external id and restore the previous mapping version pointer.

## Phase 3: Cutover
- Parallel-run window: 14 calendar days with PSP still authoritative, Oyatie shadow-authorizing disabled, webhook mirror enabled, and settlement totals reconciled daily by merchant account/currency.
- Named regression-check process: payments-migration-braintree-regression-pack: auth/capture/refund/dispute/settlement replay, idempotency collision scan, and PCI redaction audit.
- Named go/no-go gate: Go when net settled amount delta is <=0.25% by currency, unsettled authorization mismatch is 0 for active captures, webhook replay loss is 0, and PCI token handling signoff is complete.
- Cutover checkpoint 1: source freeze owner confirms freeze window and active integrations are inventoried.
- Cutover checkpoint 2: target importer version, schema digest, and mapping version are frozen.
- Cutover checkpoint 3: source and target dashboards are compared from independent read paths.
- Cutover checkpoint 4: tenant communications and rollback bridge are active.
- Rollback trigger: any P0/P1 data-loss signal, unresolved regulated-data policy gap, or failed executive gate metric.
- Rollback path: restore source authority, disable target writes, replay source deltas into staging, and reopen parallel run after root cause is fixed.
### Cutover Runbook
- Cutover action 1: execute Braintree to Oyatie payments action lane 1 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.1` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 1 and restore the previous authority flag before retrying.
- Cutover action 2: execute Braintree to Oyatie payments action lane 2 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.2` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 2 and restore the previous authority flag before retrying.
- Cutover action 3: execute Braintree to Oyatie payments action lane 3 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.3` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 3 and restore the previous authority flag before retrying.
- Cutover action 4: execute Braintree to Oyatie payments action lane 4 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.4` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 4 and restore the previous authority flag before retrying.
- Cutover action 5: execute Braintree to Oyatie payments action lane 5 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.5` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 5 and restore the previous authority flag before retrying.
- Cutover action 6: execute Braintree to Oyatie payments action lane 6 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.6` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 6 and restore the previous authority flag before retrying.
- Cutover action 7: execute Braintree to Oyatie payments action lane 7 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.7` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 7 and restore the previous authority flag before retrying.
- Cutover action 8: execute Braintree to Oyatie payments action lane 8 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.8` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 8 and restore the previous authority flag before retrying.
- Cutover action 9: execute Braintree to Oyatie payments action lane 9 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.9` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 9 and restore the previous authority flag before retrying.
- Cutover action 10: execute Braintree to Oyatie payments action lane 10 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.10` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 10 and restore the previous authority flag before retrying.
- Cutover action 11: execute Braintree to Oyatie payments action lane 11 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.11` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 11 and restore the previous authority flag before retrying.
- Cutover action 12: execute Braintree to Oyatie payments action lane 12 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.12` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 12 and restore the previous authority flag before retrying.
- Cutover action 13: execute Braintree to Oyatie payments action lane 13 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.13` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 13 and restore the previous authority flag before retrying.
- Cutover action 14: execute Braintree to Oyatie payments action lane 14 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.14` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 14 and restore the previous authority flag before retrying.
- Cutover action 15: execute Braintree to Oyatie payments action lane 15 with release-manager timestamping.
  - Expected state: exactly one authority flag changes or one reconciliation proof is attached; no untracked mutation is allowed.
  - Observable signal: `migration.cutover.payments.15` includes tenant, vendor, phase, status, delta_count, and rollback_pointer.
  - Rollback: use the rollback_pointer from action 15 and restore the previous authority flag before retrying.

## Phase 4: Verification
- Named test suite: payments-migration-braintree-regression-suite.
- Named SLO targets: P95 charge lookup <100 ms, P95 refund command <180 ms, webhook replay lag <5 minutes, settlement delta <=0.25%, and duplicate idempotency writes exactly 0.
- Named delta-detection algorithm: PSP event ledger watermark plus settlement batch Merkle hash keyed by tenant/merchant/currency/reference/eventTime and late-arrival reconciliation window.
- Verification must be run from an operator account that is not the migration writer account.
- Verification evidence includes source sample, target sample, checksum, dashboard diff, policy diff, and import log digest.
- Verification check 1: cardinality parity.
  - Method: run payments-migration-braintree-regression-suite module `cardinality_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 2: field hash parity.
  - Method: run payments-migration-braintree-regression-suite module `field_hash_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 3: relationship graph parity.
  - Method: run payments-migration-braintree-regression-suite module `relationship_graph_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 4: status/lifecycle parity.
  - Method: run payments-migration-braintree-regression-suite module `status/lifecycle_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 5: owner/principal parity.
  - Method: run payments-migration-braintree-regression-suite module `owner/principal_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 6: permission/policy parity.
  - Method: run payments-migration-braintree-regression-suite module `permission/policy_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 7: attachment checksum parity.
  - Method: run payments-migration-braintree-regression-suite module `attachment_checksum_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 8: late delta replay.
  - Method: run payments-migration-braintree-regression-suite module `late_delta_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 9: dashboard/report parity.
  - Method: run payments-migration-braintree-regression-suite module `dashboard/report_parity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 10: audit-event continuity.
  - Method: run payments-migration-braintree-regression-suite module `audit-event_continuity` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 11: SLO replay.
  - Method: run payments-migration-braintree-regression-suite module `SLO_replay` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.
- Verification check 12: rollback drill.
  - Method: run payments-migration-braintree-regression-suite module `rollback_drill` against frozen source and target snapshots.
  - Pass rule: target delta is within the named gate and every exception has a source id, target id, owner, and disposition.
  - Recovery: quarantine the failing object family, patch mapping or extractor, replay from checkpoint, and rerun this check only plus its dependencies.

## Phase 5: Decommission
- Named retention policy: Retain PSP raw reports for 7 years, PCI-scoped token evidence under token-retention policy, webhook payload ledger for 25 months, and settlement proofs for finance retention.
- Named teardown sequence: Disable PSP webhooks after final replay, revoke API keys, freeze merchant-account writes, archive settlement reports, remove PCI-scoped export credentials, and keep dispute portal read-only until tail risk closes.
- Decommission is not allowed until verification has two consecutive green windows and support has no open P1 migration incidents.
- Keep read-only source access long enough to satisfy support, legal hold, and finance/customer audit requirements.
- Archive migration bundle with schema manifest, mapping version, source checksum ledger, target checksum ledger, go/no-go signoff, and rollback drill result.
- Teardown step 1: disable writes for Braintree.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 2: archive exports for Braintree.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 3: revoke credentials for Braintree.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 4: turn off webhooks/connectors for Braintree.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 5: retire scheduled jobs for Braintree.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 6: remove temporary network access for Braintree.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 7: close support bridge for Braintree.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.
- Teardown step 8: publish final evidence for Braintree.
  - Guardrail: confirm no unresolved delta or active legal hold depends on this access before execution.
  - Rollback: restore only the minimal read-only or connector permission needed for support; do not re-enable broad write authority.

## Specific Failure Modes
- Failure 1: Webhook arrives before payment repair read.
  - Detection: payments-migration-braintree-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 2: Settlement report total differs from transaction ledger.
  - Detection: payments-migration-braintree-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 3: Vault token cannot be exported under PCI policy.
  - Detection: payments-migration-braintree-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 4: Dispute status changes during cutover freeze.
  - Detection: payments-migration-braintree-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 5: Merchant account currency mapping missing.
  - Detection: payments-migration-braintree-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 6: Refund references original transaction differently than charge.
  - Detection: payments-migration-braintree-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 7: Risk decision unavailable for legacy transaction.
  - Detection: payments-migration-braintree-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
  - Immediate recovery: stop the affected lane, leave unrelated lanes running, restore source authority for that object if authority had shifted, and open a checkpoint note.
  - Durable recovery: correct extractor/mapping/policy, replay from last green checkpoint, rerun dependent regression modules, and attach evidence before cutover resumes.
  - Escalation: service owner and vendor admin review if the same failure repeats twice or affects regulated records.
- Failure 8: Rate limit stalls repair pass.
  - Detection: payments-migration-braintree-regression-suite emits a red signal with vendor id, target id, object family, cursor, and checksum context.
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
- https://developer.paypal.com/braintree/articles/get-started/data-migration/exports/
- https://developer.paypal.com/braintree/docs/reference/general/searching/search-results
- https://developer.paypal.com/braintree/docs/guides/disputes/managing/
- docs/decisions/ADR-0116-retire-external-agent-coordination-tooling.md
- docs/decisions/ADR-0212-buildability-doctrine.md
- docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
- docs/decisions/ADR-0258-api-versioning-model.md
- docs/decisions/ADR-0263-observability-emission-contract.md

## Checkpoint
- Checkpoint bundle: migration-playbooks-w2-2026-05-20 / payments / Braintree.
- Halt condition: playbook authored, required sections present, mapping table >=30 rows, line-count floor met, and no prior-wave playbook edited.
- Next allowed action: external review or implementation planning after VCS verify/done/promote gates pass.
