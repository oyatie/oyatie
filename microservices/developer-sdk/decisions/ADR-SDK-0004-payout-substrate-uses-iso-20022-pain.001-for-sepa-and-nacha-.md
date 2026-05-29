---
id: ADR-SDK-0004
title: "Developer payout substrate uses ISO 20022, NACHA, and in-house adapters"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
related_oyatie_adrs:
  - ADR-0131
  - ADR-0173
  - ADR-0213
  - ADR-0243
  - ADR-0244
  - ADR-0258
  - ADR-0263
decision_owner: axis-ecosystem + ops-finance + council-compliance
---

# ADR-SDK-0004: Developer payout substrate uses ISO 20022, NACHA, and in-house adapters

## Context

- The named pressure is `marketplace-developer-payout-sovereignty`.
- Developer-sdk owns the developer marketplace surface where plugin, template, API, and app developers earn payouts.
- Prior incident class `hosted-payout-saas-assumed` treated Stripe as an implementation default despite ADR-0173.
- Prior incident class `currency-file-shape-ambiguous` left SEPA, ACH, and KR bank files described as generic payout exports.
- Prior incident class `reconciliation-without-ledger-ref` allowed payout file rows to exist without a durable platform ledger id.
- ADR-0213 puts developer monetization inside Ecosystem-as-a-Service.
- ADR-0173 requires avoiding vendor lock-in for strategic financial flows.
- ADR-0244 requires payout rows to be tenant-scoped even when the payee is an external developer.
- ADR-0243 requires payout eligibility and release to be Cedar-evaluated.
- ADR-0258 requires payout APIs exposed to developers to be versioned.
- ADR-0263 requires payout decisions, file generation, and bank submission attempts to emit telemetry.
- Developer payout rails vary by region and must not collapse into a single US-centric bank file.
- EU SEPA Credit Transfer requires ISO 20022 pain.001 message shape.
- US ACH requires NACHA file format discipline.
- KR firm banking requires KFTC-style bank handoff and local retention constraints.
- Marketplace payouts must support refund clawbacks, tax holds, KYC holds, sanctions holds, and minimum-balance holds.
- Payout release must be idempotent because bank submission retries are normal.
- Payout files must be signed and tamper-evident because finance operations handle them outside the app boundary.
- Payout status must be visible in the developer portal.
- Payout failures must not silently retry forever.
- Payout adapters must be in-house, but they may call bank-owned APIs where a direct banking relationship exists.
- Hosted payout providers may be emergency bridge rails only under a separate exception ADR.

## Decision

- We choose `in-house payout adapters over standards-native bank file formats`.
- The named pattern is `payment-factory file generation plus bank-adapter boundary`, using ISO 20022 and NACHA as canonical external contracts.
- SEPA payouts emit ISO 20022 `pain.001.001.09` XML.
- US ACH payouts emit NACHA file format using 94-character records.
- KR payouts emit KFTC firm-banking compatible transfer instructions through the KR pack adapter.
- Every payout row has `payout_id`, `developer_account_id`, `tenant_id`, `ledger_entry_id`, `currency`, `amount_minor`, `rail`, and `release_state`.
- Every payout row starts in `pending_clearance`.
- Release states are `pending_clearance`, `held`, `approved`, `file_generated`, `submitted`, `settled`, `returned`, `cancelled`.
- Payouts below minimum thresholds roll forward to the next payout window.
- Default minimum threshold is USD 25 equivalent.
- Default payout window is weekly.
- High-risk developers use monthly payout window until 90 days of clean history.
- Marketplace disputes can hold related payout rows for up to 180 days.
- KYC incomplete status blocks payout release.
- Sanctions hit blocks payout release until compliance clearance.
- Tax form missing status blocks payout release above jurisdictional thresholds.
- Payout file generation is idempotent by `payout_batch_id`.
- A payout row can belong to exactly one submitted bank file.
- A replacement file must reference the original `payout_batch_id`.
- Production bank file signing uses OpenBao transit key `developer-sdk-payout-file-ed25519-v1`.
- Payout release requires Cedar action `developer-sdk.payout.release`.
- Payout file generation requires Cedar action `developer-sdk.payout.file.generate`.
- Payout cancellation requires Cedar action `developer-sdk.payout.cancel`.
- Payout read requires Cedar action `developer-sdk.payout.read`.
- Payout release requires dual approval above USD 10,000 equivalent.
- Payout release requires compliance approval on any sanctions, KYC, or tax hold.
- API p95 for developer payout status read is 150 ms.
- Batch generation p95 for 100,000 payouts is 90 seconds.
- Bank submission retry window is 24 hours.
- Settlement reconciliation lag target is p95 <= 1 business day.

## Alternatives Considered

### Stripe hosted payouts

- Pro: fast implementation.
- Pro: mature KYC and tax support.
- Pro: bank integrations are handled by the vendor.
- Con: violates ADR-0173 for a strategic marketplace rail.
- Con: per-transaction fees compound at scale.
- Con: pack-specific sovereignty and residency controls are weaker.
- Con: payout failure evidence lives outside oyatie's audit chain.
- Tradeoff: speed versus ownership.
- Rejected as canonical.

### Adyen for Platforms

- Pro: global payout network.
- Pro: enterprise-grade marketplace support.
- Pro: strong EU banking footprint.
- Con: still externalizes the core payout substrate.
- Con: contract terms and data residency vary by country.
- Con: developer portal status would depend on vendor states.
- Con: custom tax and ledger integration remains necessary.
- Tradeoff: broad coverage but vendor coupling.
- Rejected as canonical; may be assessed as temporary regional bridge by exception ADR.

### Bank API only, no file generation

- Pro: modern real-time submission.
- Pro: fewer batch-file operations.
- Pro: easier status polling where banks support it.
- Con: bank APIs are inconsistent across jurisdictions.
- Con: NACHA and ISO 20022 files remain required by many banking relationships.
- Con: file-based reconciliation is still the finance audit norm.
- Tradeoff: modern UX but incomplete regional coverage.
- Rejected as sole path; bank APIs can be adapters behind the same payout batch model.

### Manual finance export

- Pro: easiest first implementation.
- Pro: finance team can inspect files before sending.
- Pro: no bank integration needed at first.
- Con: high error risk.
- Con: weak audit linkage.
- Con: no developer portal status fidelity.
- Con: does not meet hyperscaler-grade automation bar.
- Tradeoff: low engineering cost but operational fragility.
- Rejected.

## Consequences

- Positive: core payout semantics are owned by oyatie.
- Positive: standards-native files fit banking audits and finance operations.
- Positive: every payout has ledger, tenant, developer, tax, and compliance linkage.
- Positive: payout release can be controlled through Cedar and audited per ADR-0263.
- Positive: regional bank adapters can be added without changing developer portal semantics.
- Negative: implementation burden is higher than hosted payout SaaS.
- Negative: finance operations must own bank relationship setup per region.
- Negative: format validation for NACHA and ISO 20022 must be maintained.
- Negative: reconciliation is a first-class product surface, not a vendor dashboard.
- Neutral: bank-owned APIs remain usable behind in-house adapters.
- Neutral: emergency bridge rails require separate exception governance.
- Follow-up work: implement `SDK-IP-004-payout-ledger-kernel`.
- Follow-up work: implement `SDK-IP-005-iso20022-pain001-adapter`.
- Follow-up work: implement `SDK-IP-006-nacha-ach-adapter`.
- Follow-up work: add payout reconciliation dashboard and runbook.

## Implementation Notes

- Data shape `DeveloperPayoutV1` is the canonical payout row.
- Field `payout_id` is ULID prefixed by `pout_`.
- Field `developer_account_id` references the developer-sdk account.
- Field `tenant_id` is the marketplace tenant scope.
- Field `ledger_entry_id` references finance ledger.
- Field `payout_batch_id` is nullable until batch assignment.
- Field `currency` is ISO 4217 uppercase.
- Field `amount_minor` is signed 64-bit integer.
- Field `rail` is `sepa_credit_transfer`, `us_ach_credit`, `kr_firm_banking`, or `manual_exception`.
- Field `release_state` is an enum with the states named above.
- Field `hold_reasons` is an array of `kyc_incomplete`, `sanctions_review`, `tax_form_missing`, `dispute_reserve`, `minimum_threshold`, or `manual_hold`.
- Field `withholding_minor` records tax withholding.
- Field `net_amount_minor` equals gross minus withholding and fees.
- Field `idempotency_key` is unique per payout release attempt.
- Data shape `PayoutBatchV1` records generated files.
- Field `batch_id` is ULID prefixed by `pbat_`.
- Field `rail` is one payout rail.
- Field `file_digest` is SHA-256 over generated bytes.
- Field `file_signature` is Ed25519 signature.
- Field `submitted_at` is nullable until bank handoff.
- Field `settlement_due_date` is computed by rail and bank holiday calendar.
- API endpoint `GET /v1/developer/payouts` lists developer-visible payouts.
- API endpoint `GET /v1/developer/payouts/{payout_id}` returns one payout.
- API endpoint `POST /v1/developer/payouts/{payout_id}/cancel` requests cancellation before submission.
- API endpoint `POST /v1/internal/developer-payouts/batches` generates a batch.
- API endpoint `POST /v1/internal/developer-payouts/batches/{batch_id}/submit` submits to bank adapter.
- API endpoint `POST /v1/internal/developer-payouts/batches/{batch_id}/reconcile` applies bank returns.
- Cedar principal for developer reads is `DeveloperSdk::DeveloperAccount`.
- Cedar principal for finance operations is `Oyatie::Principal::Service("developer-sdk.payout-worker")`.
- Cedar action `developer-sdk.payout.read` applies to `DeveloperSdk::Payout`.
- Cedar action `developer-sdk.payout.release` applies to `DeveloperSdk::Payout`.
- Cedar action `developer-sdk.payout.file.generate` applies to `DeveloperSdk::PayoutBatch`.
- Cedar action `developer-sdk.payout.reconcile` applies to `DeveloperSdk::PayoutBatch`.
- Cedar context field `kyc_status` must be `verified`.
- Cedar context field `sanctions_status` must be `clear`.
- Cedar context field `tax_form_status` must be `complete` when jurisdiction threshold is crossed.
- Cedar context field `amount_minor_usd_equivalent` drives dual approval at 1,000,000 cents.
- Example permit: principal `developer-sdk.payout-worker`, action `developer-sdk.payout.release`, resource `DeveloperSdk::Payout::"pout_01HY"`, context `{kyc_status:"verified", sanctions_status:"clear", tax_form_status:"complete", amount_minor_usd_equivalent:500000}`.
- Example forbid: same action with context `{sanctions_status:"potential_match"}`.
- ISO 20022 files use `pain.001.001.09`.
- SEPA creditor/debtor identifiers are validated before file generation.
- NACHA files use file header, batch header, entry detail, addenda when needed, batch control, and file control records.
- NACHA record length is exactly 94 characters.
- KR firm-banking adapter enforces local bank code registry.
- Generated files are stored encrypted with per-pack KMS keys.
- Generated files retain for 7 years.
- OpenTelemetry span `developer_sdk.payout.batch_generate` wraps file generation.
- Metric `oya_developer_sdk_payout_batch_generation_seconds` tracks batch generation latency.
- Metric `oya_developer_sdk_payout_released_total` counts released payouts by rail and pack.
- Metric `oya_developer_sdk_payout_returned_total` counts bank returns by reason.
- Metric `oya_developer_sdk_payout_hold_total` counts held payouts by hold reason.
- Dashboard `developer-sdk-payouts-finance.json` shows releases, holds, returns, reconciliation lag, and bank file status.
- SLO `developer-sdk-payout-status-read.openslo.yaml` sets p95 <= 150 ms.
- SLO `developer-sdk-payout-reconciliation-lag.openslo.yaml` sets p95 <= 1 business day.
- Failure mode `bank_file_rejected` marks batch `returned` and blocks automatic resubmit until reason parsed.
- Failure mode `duplicate_submission` uses idempotency key to suppress duplicate bank handoff.
- Failure mode `sanctions_late_hit` moves unpaid payouts to `held` and emits Sev-2 compliance alert.
- Failure mode `ledger_mismatch` blocks batch generation and opens Sev-1 finance incident.
- Failure mode `file_signature_invalid` blocks submission.

## Verification

- Test `payout_release_requires_verified_kyc` verifies Cedar blocks unverified developers.
- Test `payout_release_blocks_sanctions_hit` verifies sanctions matches hold payouts.
- Test `payout_release_requires_tax_form_above_threshold` verifies tax thresholds.
- Test `payout_dual_approval_above_10000_usd` verifies approval count.
- Test `iso20022_pain001_schema_valid` validates XML against ISO 20022 schema.
- Test `nacha_record_length_exactly_94` validates ACH output.
- Test `nacha_file_control_totals_match_entries` validates batch totals.
- Test `kr_firm_banking_bank_code_registry_valid` validates KR rail.
- Test `payout_file_signature_verifies` validates Ed25519 file signatures.
- Test `payout_idempotency_prevents_duplicate_submission` verifies retry behavior.
- Metric `oya_developer_sdk_payout_batch_generation_seconds` must meet p95 <= 90 seconds for 100,000 rows.
- Metric `oya_developer_sdk_payout_returned_total` pages on return-rate > 2 percent over 1 day.
- Metric `oya_developer_sdk_payout_hold_total{reason="sanctions_review"}` pages compliance at any non-zero value.
- Dashboard `developer-sdk-payouts-finance.json` must show batch file digests and submission state.
- Dashboard `developer-marketplace-revenue.json` must join payout state to developer earnings.
- CI check `payout-iso20022-schema` validates generated SEPA fixtures.
- CI check `payout-nacha-format` validates generated ACH fixtures.
- CI check `payout-cedar-coverage` verifies payout endpoints map to Cedar actions.
- CI check `payout-ledger-link-required` rejects payout rows without ledger entry id.
- CI check `oya-governance-observability-emission --microservice developer-sdk` verifies ADR-0263 telemetry.
- Load test generates a 100,000-row SEPA batch and requires p95 <= 90 seconds.
- Reconciliation test applies bank return file and verifies payout states.
- Chaos test simulates bank submission timeout and verifies idempotent retry.
- Security test attempts production submission with unsigned file and expects refusal.
- Audit query verifies every submitted payout file has `DeveloperPayoutBatchSubmitted` event.

## References

- ADR-0131: Per-microservice flat layout.
- ADR-0173: Vendor lock-in avoidance and stack ownership.
- ADR-0213: Ecosystem-as-a-Service architecture.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0258: API versioning model.
- ADR-0263: Observability emission contract.
- ISO 20022 `pain.001.001.09` Customer Credit Transfer Initiation.
- NACHA Operating Rules and Guidelines.
- SEPA Credit Transfer Rulebook.
- KFTC firm-banking interface guidance.
- OFAC sanctions screening guidance.
- FATF Recommendation 16 wire-transfer guidance.
- SOC 2 CC6 and CC7 control criteria.
- PCI DSS v4.0 scoping guidance for payment-adjacent data.
- OpenBao transit documentation.
- RFC 8032: Ed25519 signatures.
