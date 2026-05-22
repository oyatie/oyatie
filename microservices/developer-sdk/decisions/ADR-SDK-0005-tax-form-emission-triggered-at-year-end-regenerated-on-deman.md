---
id: ADR-SDK-0005
title: "Developer tax forms are year-end ledger emissions with append-only regeneration"
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

# ADR-SDK-0005: Developer tax forms are year-end ledger emissions with append-only regeneration

## Context

- The named pressure is `developer-marketplace-tax-evidence`.
- Developer-sdk pays external developers for marketplace earnings and must generate tax evidence by jurisdiction.
- Prior incident class `silent-tax-form-reissue` allowed a generated form to be replaced without preserving the old hash.
- Prior incident class `gross-net-confusion` mixed gross marketplace revenue, withholding, fees, and net payouts in one field.
- Prior incident class `tax-threshold-unversioned` described 1099 and VAT thresholds as prose instead of versioned jurisdiction rules.
- ADR-0213 makes developer monetization a first-class Ecosystem-as-a-Service capability.
- ADR-0244 requires every tax form to remain tenant-scoped and developer-scoped.
- ADR-0243 requires emission, regeneration, suppression, and correction to be Cedar-gated.
- ADR-0258 requires developer-facing tax APIs to remain versioned.
- ADR-0263 requires every form emission and regeneration to produce audit evidence.
- US developers may receive IRS Form 1099-NEC or 1099-K depending on role, payment processor posture, and statutory threshold.
- EU developers may require VAT OSS/IOSS evidence or reverse-charge invoices depending on VAT registration.
- KR developers may require VAT invoice and withholding evidence under local tax rules.
- India developers may require TDS evidence.
- Brazil developers may require Nota Fiscal alignment through regional pack adapters.
- The platform must distinguish tax-form generation from payout execution.
- Tax forms are evidence, not mutable profile records.
- A developer may request regeneration after profile correction.
- Regeneration must produce a new immutable form version, not overwrite the old one.
- Corrections must preserve a chain from original form to corrected form.
- Tax thresholds change annually and must be effective-date versioned.
- Tax data must be retained for at least 7 years unless a stricter pack overlay applies.
- The developer portal must make current and historical forms downloadable.

## Decision

- We choose `year-end ledger-triggered tax form emission with append-only regeneration`.
- The named pattern is `append-only tax document ledger`, similar to financial statement restatement discipline.
- The emission trigger runs after fiscal year close.
- Default fiscal-year close is calendar year end.
- Tenant-specific fiscal year is allowed only when marketplace contract declares it.
- Form generation reads immutable ledger entries, not payout row summaries.
- Form generation uses `DeveloperTaxLedgerEntryV1`.
- Form generation creates `DeveloperTaxFormV1`.
- Every form version gets a unique `tax_form_version_id`.
- The first generated form has `version=1`.
- Regenerated forms increment `version`.
- Corrected forms carry `correction_of_tax_form_version_id`.
- No form version is deleted.
- A form may be marked `voided`, but bytes and digest remain retained.
- Developer-requested regeneration is allowed only for profile corrections, tax-id corrections, or address corrections.
- Finance-requested correction is allowed for ledger correction, withholding correction, or statutory form update.
- Developer-requested regeneration requires Cedar action `developer-sdk.tax_form.regenerate`.
- Finance correction requires Cedar action `developer-sdk.tax_form.correct`.
- Form download requires Cedar action `developer-sdk.tax_form.read`.
- Form suppression requires Cedar action `developer-sdk.tax_form.suppress`.
- US 1099 evidence generation supports 1099-NEC and 1099-K profiles.
- EU VAT evidence supports VAT OSS/IOSS summary and reverse-charge invoice profile.
- KR evidence supports VAT and withholding statement profile.
- India evidence supports TDS statement profile.
- Brazil evidence is an integration profile that requires regional pack adapter.
- Form bytes are PDF/A-3 plus canonical JSON sidecar.
- Canonical JSON sidecar is RFC 8785 serialized.
- Form digest is SHA-256 over the canonical JSON sidecar and PDF bytes.
- Form signature is Ed25519 via OpenBao transit.
- Public API exposes form status and download links; internal API generates and corrects forms.
- Year-end emission target is p95 <= 24 hours after ledger close.
- On-demand regeneration target is p95 <= 10 minutes.
- Download API p95 target is 200 ms excluding object-storage transfer.
- Form audit coverage target is 100 percent.

## Alternatives Considered

### Silently replace regenerated forms

- Pro: simplest user experience.
- Pro: developer always sees one current file.
- Pro: fewer records to store.
- Con: forensic integrity is broken.
- Con: tax authority disputes cannot reconstruct what was previously issued.
- Con: support cannot tell whether a developer downloaded the old or new version.
- Con: contradicts audit-chain doctrine.
- Tradeoff: UX simplicity but unacceptable evidence loss.
- Rejected.

### Generate tax forms at every payout

- Pro: near-real-time evidence.
- Pro: easier to reason about single payout-to-form mapping.
- Pro: fewer year-end batch spikes.
- Con: most jurisdictions require annual summaries or period summaries.
- Con: corrections multiply across many files.
- Con: developer portal gets noisy.
- Tradeoff: operational smoothness but wrong statutory shape.
- Rejected; payout receipts remain separate from tax forms.

### Outsource all tax forms to a tax SaaS

- Pro: faster jurisdiction coverage.
- Pro: vendor handles statutory updates.
- Pro: less internal compliance work.
- Con: externalizes strategic marketplace data.
- Con: ADR-0173 stack ownership weakens.
- Con: pack residency and audit-chain coverage become vendor-dependent.
- Con: developer portal states split across vendors.
- Tradeoff: coverage speed but vendor coupling.
- Rejected as canonical; tax-rate content feeds may be licensed with pack review.

### Manual finance-generated forms

- Pro: lowest engineering cost.
- Pro: finance can handle edge cases.
- Pro: no immediate generation engine.
- Con: does not scale.
- Con: weak idempotency.
- Con: high risk of missed forms.
- Con: no API for developers.
- Tradeoff: expedient but not platform-grade.
- Rejected.

## Consequences

- Positive: tax form evidence becomes immutable and reconstructible.
- Positive: developers can see exactly when a form was generated, corrected, voided, or downloaded.
- Positive: jurisdiction rules are versioned rather than hidden in finance process.
- Positive: annual generation and on-demand regeneration share the same engine.
- Positive: audit-chain evidence supports disputes and regulator inquiries.
- Negative: form storage grows with every correction.
- Negative: jurisdiction profiles require yearly maintenance.
- Negative: developer support must explain versions and corrections.
- Negative: finance must review rule changes before year-end.
- Neutral: payout receipts remain separate operational artifacts.
- Neutral: licensed tax content can inform rules but cannot own evidence.
- Follow-up work: implement `SDK-IP-007-tax-ledger-entry-model`.
- Follow-up work: implement `SDK-IP-008-tax-form-generator`.
- Follow-up work: add tax rule effective-date registry.
- Follow-up work: add developer portal tax-form version timeline.

## Implementation Notes

- Data shape `DeveloperTaxLedgerEntryV1` is the source event for form generation.
- Field `entry_id` is ULID prefixed by `taxle_`.
- Field `developer_account_id` references developer-sdk account.
- Field `tenant_id` scopes marketplace tenant.
- Field `earning_period_start` is date.
- Field `earning_period_end` is date.
- Field `gross_amount_minor` is 64-bit integer.
- Field `fee_amount_minor` is 64-bit integer.
- Field `withholding_amount_minor` is 64-bit integer.
- Field `net_payout_amount_minor` is 64-bit integer.
- Field `currency` is ISO 4217.
- Field `jurisdiction` is ISO 3166-1 alpha-2 plus subdivision when needed.
- Field `tax_profile` is `us_1099_nec`, `us_1099_k`, `eu_vat_oss`, `kr_vat`, `india_tds`, `br_nota_fiscal`, or `manual_review`.
- Data shape `DeveloperTaxFormV1` is the emitted form.
- Field `tax_form_id` is ULID prefixed by `taxf_`.
- Field `tax_form_version_id` is ULID prefixed by `taxfv_`.
- Field `tax_year` is four-digit year.
- Field `version` starts at 1.
- Field `status` is `generated`, `corrected`, `voided`, `suppressed`, or `delivered`.
- Field `correction_of_tax_form_version_id` is nullable.
- Field `form_kind` is jurisdiction profile.
- Field `pdfa_object_ref` points to encrypted object storage.
- Field `canonical_json_object_ref` points to encrypted object storage.
- Field `form_digest_sha256` stores the digest.
- Field `signature` stores Ed25519 signature.
- Field `generated_at` is RFC 3339.
- Field `downloaded_at_last` is nullable.
- API endpoint `GET /v1/developer/tax-forms` lists developer forms.
- API endpoint `GET /v1/developer/tax-forms/{tax_form_id}` returns form metadata.
- API endpoint `GET /v1/developer/tax-forms/{tax_form_id}/download` returns short-lived signed URL.
- API endpoint `POST /v1/developer/tax-forms/{tax_form_id}/regenerate` requests regeneration.
- API endpoint `POST /v1/internal/developer-tax/year-end-run` starts annual generation.
- API endpoint `POST /v1/internal/developer-tax/forms/{tax_form_id}/correct` creates correction.
- API endpoint `POST /v1/internal/developer-tax/forms/{tax_form_id}/void` voids a form.
- Cedar principal for developer reads is `DeveloperSdk::DeveloperAccount`.
- Cedar principal for finance is `Oyatie::Principal::Service("developer-sdk.tax-worker")`.
- Cedar action `developer-sdk.tax_form.read` applies to `DeveloperSdk::TaxForm`.
- Cedar action `developer-sdk.tax_form.regenerate` applies to `DeveloperSdk::TaxForm`.
- Cedar action `developer-sdk.tax_form.correct` applies to `DeveloperSdk::TaxForm`.
- Cedar action `developer-sdk.tax_form.suppress` applies to `DeveloperSdk::TaxForm`.
- Cedar context field `developer_account_id` must match principal account for reads and regeneration.
- Cedar context field `reason` must be one of `profile_correction`, `tax_id_correction`, `address_correction`, `ledger_correction`, `withholding_correction`, or `statutory_update`.
- Cedar context field `tax_year_locked` must be false for normal generation and true only with finance override.
- Example permit: principal `DeveloperSdk::DeveloperAccount::"dev_01HY"`, action `developer-sdk.tax_form.regenerate`, resource `DeveloperSdk::TaxForm::"taxf_01HY"`, context `{reason:"address_correction", account_matches:true}`.
- Example forbid: same principal and action with context `{reason:"ledger_correction"}`.
- OpenBao key path is `transit/keys/{cell_id}/{pack_id}/developer-sdk/tax-form-ed25519-v1`.
- PDF/A-3 generation embeds canonical JSON as attachment where jurisdiction permits.
- Object storage uses per-pack KMS keys.
- Download URLs expire after 10 minutes.
- Audit event `DeveloperTaxFormGenerated` emits on generation.
- Audit event `DeveloperTaxFormCorrected` emits on correction.
- Audit event `DeveloperTaxFormVoided` emits on void.
- Audit event `DeveloperTaxFormDownloaded` emits on download.
- OpenTelemetry span `developer_sdk.tax_form.generate` wraps generation.
- Metric `oya_developer_sdk_tax_forms_generated_total` counts forms by jurisdiction and kind.
- Metric `oya_developer_sdk_tax_form_generation_seconds` tracks generation runtime.
- Metric `oya_developer_sdk_tax_form_regeneration_seconds` tracks regeneration runtime.
- Metric `oya_developer_sdk_tax_form_correction_total` counts corrections by reason.
- Dashboard `developer-sdk-tax-forms.json` shows year-end progress, failures, corrections, downloads, and jurisdiction mix.
- SLO `developer-sdk-tax-year-end-emission.openslo.yaml` sets p95 <= 24 hours after close.
- SLO `developer-sdk-tax-regeneration.openslo.yaml` sets p95 <= 10 minutes.
- Failure mode `rule_missing_for_jurisdiction` blocks generation and opens compliance ticket.
- Failure mode `ledger_mismatch` blocks form and opens finance Sev-1.
- Failure mode `signature_failed` blocks delivery.
- Failure mode `developer_tax_id_invalid` moves form to `suppressed` with visible remediation.
- Failure mode `download_url_expired` returns 410 and allows retry.

## Verification

- Test `tax_form_generation_uses_ledger_entries` verifies forms cannot be generated from payout summaries alone.
- Test `tax_form_regeneration_creates_new_version` verifies no overwrite.
- Test `tax_form_correction_links_original` verifies correction chain.
- Test `tax_form_digest_changes_on_correction` verifies hash evidence.
- Test `tax_form_signature_verifies` validates Ed25519 signature.
- Test `tax_form_developer_read_scope` verifies developers cannot read other accounts.
- Test `tax_form_finance_correction_scope` verifies only finance principal can correct ledger issues.
- Test `tax_form_download_url_expires` verifies 10-minute URL TTL.
- Test `tax_form_year_end_idempotency` verifies repeated run does not duplicate versions.
- Test `tax_form_void_preserves_bytes` verifies voiding retains object references.
- Metric `oya_developer_sdk_tax_form_generation_seconds` must meet year-end p95 target.
- Metric `oya_developer_sdk_tax_form_regeneration_seconds` must meet 10-minute p95 target.
- Metric `oya_developer_sdk_tax_form_correction_total` must be visible by reason.
- Dashboard `developer-sdk-tax-forms.json` must show jurisdiction gaps before year-end.
- Dashboard `finance-compliance-year-end.json` must link form status to payout ledger.
- CI check `tax-form-schema-fixtures` validates JSON sidecars.
- CI check `tax-form-pdfa-generation` validates PDF/A-3 output.
- CI check `tax-form-cedar-coverage` verifies API-to-Cedar mapping.
- CI check `tax-form-rule-effective-date` verifies jurisdiction rules have effective dates.
- CI check `oya-governance-observability-emission --microservice developer-sdk` verifies ADR-0263 telemetry.
- Batch test generates 100,000 tax forms and requires completion under 24-hour SLO projection.
- Regeneration test creates corrected address and verifies version 2 is linked to version 1.
- Security test attempts download by another developer and expects 403.
- Audit query verifies generation count equals audit event count.
- Retention test verifies generated forms retain for 7 years.

## References

- ADR-0131: Per-microservice flat layout.
- ADR-0173: Vendor lock-in avoidance and stack ownership.
- ADR-0213: Ecosystem-as-a-Service architecture.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0258: API versioning model.
- ADR-0263: Observability emission contract.
- IRS Instructions for Forms 1099-MISC and 1099-NEC.
- IRS Form 1099-K instructions.
- EU VAT One Stop Shop guidance.
- Korean VAT Act and withholding evidence guidance.
- India Income Tax Act TDS guidance.
- Brazil Nota Fiscal electronic invoicing guidance.
- PDF/A-3 ISO 19005-3.
- RFC 8785: JSON Canonicalization Scheme.
- RFC 8032: Ed25519 signatures.
- SOC 2 CC6.1 and CC7.2.
- ISO/IEC 27001:2022 A.5.33 records protection.
