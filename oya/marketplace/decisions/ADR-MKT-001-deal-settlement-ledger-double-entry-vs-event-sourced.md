---
id: ADR-MKT-001
title: Deal Settlement Ledger Double Entry vs Event Sourced
status: Proposed
date: 2026-05-20
microservice: marketplace
related_oyatie_adrs:
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0704-k8s-port-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
decision_owner: axis-marketplace
---

# ADR-MKT-001: Deal Settlement Ledger Double Entry vs Event Sourced

## Context

- Marketplace is the universal deal-settlement substrate per ADR-0314.
- The j103 implementation plan names marketplace as the deal-settlement-ledger owner for just-in-time procurement.
- DealSet covers offers, acceptance, obligations, entitlements, settlement, disputes, amendments, revocation, and renewals.
- Stripe is a named precedent for platform and marketplace multiparty payments.
- Shopify Plus B2B is a named precedent for enterprise commerce terms, companies, catalogs, and payment terms.
- Settlement state must feed payments, treasury, finops-portal, workflow-engine, ontology, audit-chain, and compliance.
- Named pressure MKT-P1: marketplace needs accounting-grade balance invariants for money and entitlement movement.
- Named pressure MKT-P2: marketplace needs replayable event history for disputes and audit-chain reconstruction.
- Named pressure MKT-P3: every commercial transition is tenant-scoped and Cedar-gated.
- Named pressure MKT-P4: idempotency is mandatory because acceptance, escrow reservation, and settlement can retry.
- Named pressure MKT-P5: ERP parity requires non-money obligations and entitlement grants alongside money movement.
- Named pressure MKT-P6: DealSet imports from external systems need provenance and reversible staging.
- Named pressure MKT-P7: settlement rails can be synchronous for internal credits and asynchronous for PSP or bank rails.
- Named pressure MKT-P8: chargebacks, refunds, disputes, sanctions holds, and tax holds require compensating entries.
- Named pressure MKT-P9: tenants need ledger queries without scanning raw event streams.
- Named pressure MKT-P10: audit evidence must explain both what happened and why balances still reconcile.
- Constraint MKT-C1: ADR-0314 makes DealSet the universal settlement envelope.
- Constraint MKT-C2: ADR-0249 keeps marketplace multi-category rather than retail-only.
- Constraint MKT-C3: ADR-0244 requires tenant scoping on every row and event.
- Constraint MKT-C4: ADR-0243 requires Cedar gates for offer, acceptance, settlement, dispute, and entitlement actions.
- Constraint MKT-C5: ADR-0149 requires idempotency keys for mutation retries.
- Constraint MKT-C6: ADR-0153 requires outbox emission for reliable events.
- Constraint MKT-C7: ADR-0154 requires event schema versioning.
- Constraint MKT-C8: ADR-0222 requires compensation for sagas.
- Constraint MKT-C9: ADR-0174 requires FinOps and chargeback-ready cost attribution.
- Constraint MKT-C10: ADR-0003 requires audit-chain evidence for settlement transitions.
- Double-entry ledgers provide strong balance invariants.
- Event sourcing provides rich history and replay.
- A pure double-entry table can hide causal workflow.
- A pure event stream can make current balances and reconciliation expensive.
- This ADR chooses a hybrid with strict ownership boundaries.

## Decision

- Adopt a double-entry settlement ledger as the canonical balance state.
- Adopt event-sourced transition history as the canonical causal history.
- Name the hybrid `DealSettlementLedger v1`.
- Store every monetary, credit, escrow, entitlement, tax, fee, refund, and chargeback movement as balanced ledger entries.
- Store every DealSet lifecycle transition as an append-only domain event.
- Link each ledger entry batch to exactly one domain event or one compensating event.
- Require every ledger batch to balance per currency and per entitlement class.
- Require every ledger batch to include tenant_id, deal_set_id, counterparty role, idempotency key, and audit_event_id.
- Use event history for dispute reconstruction.
- Use double-entry tables for current balance, escrow, receivable, payable, and entitlement queries.
- Use outbox events to publish settlement transitions after database commit.
- Use saga compensation for PSP failure, sanctions hold, tax hold, refund, and chargeback.
- Use a settlement journal for immutable entry batches.
- Use read models for per-tenant receivables, payable exposure, escrow balances, and entitlement grants.
- Use Cedar to gate transition commands before events or ledger entries are created.
- Use audit-chain to seal command intent, permit id, event id, ledger batch id, and resulting balance hash.
- Use Stripe as a precedent for marketplace payout and connected account concepts, not as the system of record.
- Use Shopify Plus B2B as a precedent for company terms and B2B purchasing flows, not as the settlement ledger.
- Keep PSP authorization and capture in payments.
- Keep treasury cash position in treasury.
- Keep invoice presentation and chargeback analytics in finops-portal.
- Keep DealSet lifecycle and settlement intent in marketplace.
- Keep money movement rail execution outside marketplace but record its settlement result in marketplace ledger.
- Require external import batches to land in `staged_import` before ledger posting.
- Require replay verification to rebuild ledger balances from events in CI and scheduled production checks.
- Require ledger reconciliation to run per tenant, per counterparty, per currency, and per day.
- Name event `marketplace.deal.accepted.v1`.
- Name event `marketplace.settlement.ledger_batch_posted.v1`.
- Name event `marketplace.settlement.compensation_posted.v1`.
- Name event `marketplace.settlement.reconciliation_failed.v1`.
- Make this ADR authoritative for marketplace DealSet settlement state.

## Alternatives Considered

### Pure Double-Entry Ledger

- Pros: excellent balance integrity.
- Pros: familiar accounting reconciliation model.
- Pros: efficient current balance and exposure queries.
- Cons: weak causal reconstruction unless every workflow state is duplicated in entries.
- Cons: disputes need full event context, not just debits and credits.
- Cons: lifecycle transitions like offer acceptance and entitlement mutation are not naturally accounting entries.
- Rejected as the sole model; accepted as canonical balance state.

### Pure Event Sourcing

- Pros: complete history of commands and transitions.
- Pros: natural replay for disputes and audit.
- Pros: easy to add new projections.
- Cons: balance queries require projection correctness.
- Cons: accounting invariants can become eventual rather than immediate.
- Cons: reconciliation against PSP, tax, and treasury systems becomes harder.
- Rejected as the sole model; accepted as canonical causal history.

### Payments Provider Ledger as Source of Truth

- Pros: Stripe and PSPs already model charges, payouts, refunds, and disputes.
- Pros: lower initial accounting build.
- Pros: external reconciliation tools exist.
- Cons: violates ADR-0314 universal settlement beyond money.
- Cons: cannot model non-money obligations and entitlements consistently.
- Cons: provider outages or account models become product architecture.
- Rejected because marketplace must be first-party and cross-rail.

### One Ledger Table Per Deal Category

- Pros: category-specific fields are easy.
- Pros: simpler first implementation for retail orders.
- Pros: domain teams can move independently.
- Cons: breaks universal settlement semantics.
- Cons: duplicates compensation and reconciliation logic.
- Cons: makes cross-category ERP parity hard.
- Rejected because ADR-0314 requires universal DealSet settlement.

### Blockchain or Distributed Ledger

- Pros: append-only and externally verifiable.
- Pros: attractive for multiparty settlement narratives.
- Pros: can provide shared state across counterparties.
- Cons: unnecessary for internal tenant-scoped settlement.
- Cons: privacy, residency, and erasure tensions are high.
- Cons: operational complexity exceeds current value.
- Rejected for v1; audit-chain already provides tamper-evident evidence.

## Consequences

- Positive: current balances are fast and accounting-grade.
- Positive: causal history is replayable for disputes.
- Positive: DealSet remains universal across goods, services, subscriptions, workforce contracts, and data licenses.
- Positive: PSP integrations can fail and compensate without corrupting ledger state.
- Positive: tenants can query receivable, payable, escrow, and entitlement positions without event scans.
- Positive: audit-chain seals both command history and balance effect.
- Positive: FinOps and ERP projections share one settlement primitive.
- Negative: two models must stay consistent.
- Negative: ledger posting and event append must use one transactional boundary.
- Negative: replay verification becomes mandatory operational work.
- Negative: engineers must model non-money entitlements as balanced movements.
- Neutral: read models can be rebuilt from events and ledger entries.
- Neutral: PSP provider ids remain external references, not primary keys.
- Neutral: treasury remains the cash-position owner.
- Neutral: payments remains the rail-execution owner.
- Follow-up work MKT-F1: add `settlement_ledger_entry` schema migration.
- Follow-up work MKT-F2: add event-to-ledger replay checker.
- Follow-up work MKT-F3: add PSP reconciliation adapter contract.
- Follow-up work MKT-F4: add dashboard for deal settlement exposure.
- Follow-up work MKT-F5: add staged import runbook for Stripe and Shopify data.

## Implementation Notes

- Data shape `DealSet`: `{tenant_id, deal_set_id, category, counterparty_roles, status, settlement_policy_id, audit_stream_id}`.
- Data shape `SettlementCommand`: `{tenant_id, command_id, deal_set_id, action, idempotency_key, requested_by, cedar_permit_id}`.
- Data shape `SettlementEvent`: `{tenant_id, event_id, deal_set_id, event_type, schema_version, command_id, occurred_at}`.
- Data shape `LedgerBatch`: `{tenant_id, ledger_batch_id, deal_set_id, event_id, balance_hash_before, balance_hash_after, posted_at}`.
- Data shape `LedgerEntry`: `{tenant_id, ledger_batch_id, account_id, side, amount, currency, entitlement_class, counterparty_role}`.
- Data shape `SettlementAccount`: `{tenant_id, account_id, account_kind, currency, entitlement_class, owner_role, normal_side}`.
- Data shape `CompensationPlan`: `{tenant_id, deal_set_id, failed_step, compensating_event_type, ledger_batch_id, reason_code}`.
- Data shape `ReconciliationRun`: `{tenant_id, run_id, period, source, expected_hash, observed_hash, mismatch_count}`.
- Postgres table `marketplace_deal_set` stores DealSet envelopes.
- Postgres table `marketplace_settlement_event` stores append-only events.
- Postgres table `marketplace_ledger_batch` stores posting batches.
- Postgres table `marketplace_ledger_entry` stores balanced entries.
- Postgres table `marketplace_settlement_account` stores account definitions.
- Postgres table `marketplace_reconciliation_run` stores verification state.
- Unique constraint `(tenant_id, idempotency_key, action)` prevents duplicate command posting.
- Check constraint `ledger_batch_balances_by_currency` requires zero-sum per currency.
- Check constraint `ledger_batch_balances_by_entitlement_class` requires zero-sum per entitlement class.
- REST endpoint `POST /v1/marketplace/deal-sets/{deal_set_id}/accept` creates accepted event and ledger reservation.
- REST endpoint `POST /v1/marketplace/deal-sets/{deal_set_id}/settle` posts settlement entries.
- REST endpoint `POST /v1/marketplace/deal-sets/{deal_set_id}/refund` posts compensating refund entries.
- REST endpoint `POST /v1/marketplace/deal-sets/{deal_set_id}/disputes` opens dispute event without unbalanced mutation.
- REST endpoint `GET /v1/marketplace/deal-sets/{deal_set_id}/ledger` returns Cedar-filtered ledger view.
- REST endpoint `POST /v1/marketplace/reconciliation-runs` starts tenant reconciliation.
- AsyncAPI channel `marketplace.deal.accepted.v1` publishes acceptance.
- AsyncAPI channel `marketplace.settlement.ledger_batch_posted.v1` publishes posted batch summary.
- AsyncAPI channel `marketplace.settlement.compensation_posted.v1` publishes compensating batch.
- AsyncAPI channel `marketplace.settlement.reconciliation_failed.v1` publishes mismatch.
- Cedar action `marketplace::deal::accept` requires counterparty authority and deal status.
- Cedar action `marketplace::deal::settle` requires settlement policy and no active hold.
- Cedar action `marketplace::ledger::read` requires participant role or auditor role.
- Cedar action `marketplace::settlement::compensate` requires saga failure evidence.
- Cedar action `marketplace::import::stage` requires tenant admin and external system provenance.
- SLO target `marketplace_ledger_post_p95_ms` is <=300 for single-tenant internal settlement.
- SLO target `marketplace_ledger_balance_correctness_ratio` is 1.0.
- SLO target `marketplace_reconciliation_completion_p95_minutes` is <=30 per tenant-day.
- SLO target `marketplace_settlement_outbox_lag_p95_seconds` is <=10.
- SLO target `marketplace_duplicate_command_rejection_ratio` is 1.0.

## Verification

- Unit test `ledger_batch_must_balance_by_currency` proves monetary balance invariant.
- Unit test `ledger_batch_must_balance_by_entitlement_class` proves non-money balance invariant.
- Unit test `accept_command_requires_cedar_permit` proves policy gate.
- Unit test `duplicate_idempotency_key_replays_existing_result` proves retry safety.
- Unit test `compensation_requires_failed_saga_step` proves compensation discipline.
- Unit test `psp_reference_cannot_be_primary_key` proves provider displacement.
- Contract test `deal_ledger_endpoint_filters_counterparty_view` proves tenant and role filtering.
- Contract test `ledger_batch_posted_event_contains_balance_hashes` proves event completeness.
- Property test `random_balanced_batches_never_change_global_sum` proves ledger algebra.
- Replay test `events_rebuild_ledger_read_model` proves event history sufficiency.
- Replay test `ledger_entries_rebuild_current_exposure` proves balance state sufficiency.
- Integration test `accept_then_psp_failure_posts_compensation` proves saga behavior.
- Integration test `stripe_import_lands_in_staged_import_not_ledger` proves import discipline.
- Integration test `shopify_b2b_terms_map_to_deal_terms_not_provider_ledger` proves precedent boundary.
- Failure test `outbox_publish_failure_does_not_lose_committed_ledger_batch` proves ADR-0153 posture.
- Failure test `reconciliation_mismatch_emits_event_and_blocks_promotion` proves verification gate.
- Security test `non_participant_cannot_read_deal_ledger` proves Cedar role gate.
- Security test `counterparty_cannot_settle_without_authority` proves delegated authority.
- Metric `marketplace_ledger_post_duration_ms` tracks posting latency.
- Metric `marketplace_ledger_unbalanced_reject_total` tracks rejected invalid batches.
- Metric `marketplace_settlement_compensation_total` tracks compensation by reason.
- Metric `marketplace_reconciliation_mismatch_total` tracks mismatches.
- Metric `marketplace_outbox_lag_seconds` tracks reliable event emission.
- Metric `marketplace_duplicate_command_total` tracks idempotency replays.
- Dashboard `marketplace-deal-settlement-ledger` shows post latency, batch counts, and balance rejects.
- Dashboard `marketplace-settlement-reconciliation` shows tenant-day reconciliation health.
- Dashboard `marketplace-compensation-sagas` shows refunds, chargebacks, PSP failures, and sanctions holds.
- Dashboard `marketplace-provider-imports` shows Stripe, Shopify, and custom imports in staged state.
- Alert `MarketplaceLedgerUnbalancedAttempt` fires on any rejected unbalanced batch.
- Alert `MarketplaceReconciliationMismatch` fires on any tenant-day mismatch.
- Alert `MarketplaceOutboxLagBudget` fires when outbox p95 lag exceeds 10 seconds.
- Alert `MarketplaceDuplicateCommandAnomaly` fires when duplicate rate exceeds baseline.

## References

- Internal: docs/decisions/ADR-0705-product-protocol-live-apex.md
- Internal: docs/decisions/ADR-0705-product-protocol-live-apex.md
- Internal: docs/decisions/ADR-0709-general-live-apex.md
- Internal: docs/decisions/ADR-0709-general-live-apex.md
- Internal: docs/decisions/ADR-0704-k8s-port-live-apex.md
- Internal: microservices/marketplace/IP-journey-j103-deal-settlement-ledger.md
- Stripe documentation: https://docs.stripe.com/connect/how-connect-works
- Stripe platform and marketplace payments: https://stripe.com/connect
- Shopify B2B documentation: https://help.shopify.com/manual/b2b
- Shopify apps and B2B developer documentation: https://shopify.dev/docs/apps/selling-strategies/b2b
- Martin Fowler, Event Sourcing: https://www.martinfowler.com/eaaDev/EventSourcing.html
- Martin Fowler, What do you mean by event-driven: https://martinfowler.com/articles/201701-event-driven.html
- On Double-Entry Bookkeeping, mathematical treatment: https://arxiv.org/abs/1407.1898
- REA, Triple-Entry Accounting and Blockchain: https://arxiv.org/abs/2005.07802
- OpenAPI Specification: https://spec.openapis.org/oas/
- AsyncAPI Specification: https://www.asyncapi.com/docs/reference/specification/latest
- CloudEvents Specification: https://cloudevents.io/
- Cedar policy language syntax: https://docs.cedarpolicy.com/policies/syntax-policy.html
