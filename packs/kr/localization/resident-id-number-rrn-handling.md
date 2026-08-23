---
doc_class: LocalizationPack
pack_id: KR-PACK-1
doc_id: KR-PACK-1-RRN-HANDLING
title: Korea Localization Pack Resident Registration Number Handling
version: 1.0.0
status: canonical-draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0064
  - ADR-0244
  - ADR-0251
  - ADR-0263
citing_authority_url:
  - https://www.law.go.kr/
  - https://www.pipc.go.kr/
  - https://www.kisa.or.kr/
---

# Korea Resident Registration Number Handling

This document defines KR-PACK-1 handling for Korean Resident Registration Numbers, `주민등록번호`, abbreviated RRN.
RRN handling is a default-deny control family.
Consent alone is not sufficient to collect, store, export, or display RRN.
Statutory basis must be recorded before RRN processing.
Where identity assurance is needed, CI/DI or another approved alternative identifier is preferred.
Where a persistent join key is required, irreversible hashing is required unless raw retention is legally mandatory.

## RRN Doctrine

RRN is not an ordinary user identifier.
RRN is not a display identifier.
RRN is not a login identifier.
RRN is not a support identifier.
RRN is not an analytics identifier.
RRN is not a convenient deduplication key.
RRN is not collected by generic consent.
RRN is not exported by routine workflow.
RRN is not logged.
RRN is not copied into audit payloads.
RRN is not stored in support tickets.
RRN is not stored in search index.
RRN is not stored in BI extracts.
RRN is not stored in model-training datasets.
RRN is not stored in telemetry.
RRN is not retained after statutory need ends.
RRN is processed only under statutory basis.
RRN raw value is transient by default.
RRN derivative must be irreversible by default.
RRN derivative must be scoped by tenant and purpose.
RRN access must be exceptional and audited.

## Authority Citations

Authority snapshot date: 2026-05-20.
Primary law source: `https://www.law.go.kr/`.
Primary privacy regulator source: `https://www.pipc.go.kr/`.
Primary KISA source: `https://www.kisa.or.kr/`.
PIPA is cited as Personal Information Protection Act, `개인정보 보호법`.
PIPA Article 24 governs unique identifying information.
PIPA Article 24-2 governs resident registration number processing restrictions.
PIPA Article 15 governs collection and use.
PIPA Article 17 governs third-party provision.
PIPA Article 22 governs consent method.
PIPA Article 23 governs sensitive information when RRN appears with sensitive records.
PIPA Article 28-8 governs overseas transfer.
PIPA Article 34 governs leakage notification and reporting.
PIPA Enforcement Decree Article 40 governs report timing for enumerated leakage cases.
Information Network Act lineage is cited for non-RRN identity verification alternatives.
Information Network Act Article 23-2 lineage concerns non-RRN methods of identity verification.
Information Network Act Article 23-3 lineage concerns identity confirmation agency controls.
Telecommunications Business Act and related identity-provider practice inform CI/DI usage in Korean identity ecosystems.
Resident Registration Act source may govern source-number issuance and civil registry context.
ADR-0244 requires `tenant_id` and `sub_scope_path` as universal scoping primitives.
ADR-0251 requires compliance packs to create and enforce data classes such as `PI_KR_RESIDENT_REGISTRATION_NUMBER`.
ADR-0263 requires PII-scrubbed audit emission boundaries.
Named effective dates must be read from law.go.kr for final bundle build.
Korean official text controls over translation drift.
If PIPC guidance changes Article 24-2 handling, this document must be updated before release.

## ADR-0244 Binding

Every RRN policy decision requires `tenant_id`.
Every RRN policy decision requires `sub_scope_path` when a subject or record is scoped.
Every RRN field must be part of tenant-scoped data model.
Every RRN derivative must include tenant scope in derivation context.
Every RRN statutory basis must be scoped to tenant and purpose.
Every RRN access approval must be scoped to tenant and purpose.
Every RRN denial must include tenant context.
Every RRN audit event must include tenant context.
Every RRN audit event must include sub-scope when scoped.
Every RRN processor access decision must include tenant context.
Every RRN transfer assessment must include tenant context.
Every RRN legal hold must include tenant context.
Every RRN deletion checkpoint must include tenant context.
Every RRN migration must include tenant context.
Every RRN backup restore must include tenant context.
Every RRN support escalation must include tenant context.
Every RRN incident classification must include tenant context.
Every RRN breach report clock must include tenant context.
Every RRN Cedar entity must be tenant-scoped.
No service-local identifier may bypass ADR-0244 scoping.

## ADR-0251 Binding

RRN controls are compliance-pack controls.
RRN controls activate only when `KR-PACK-1` is installed.
RRN controls create data class `PI_KR_RRN`.
RRN controls create data class `PI_KR_RESIDENT_REGISTRATION_NUMBER`.
RRN controls create derivative class `PI_KR_RRN_HASH_DERIVATIVE`.
RRN controls create evidence class `KR_RRN_STATUTORY_BASIS`.
RRN controls create audit class `KR_RRN_ACCESS_EVIDENCE`.
RRN controls require cell certification where RRN derivative is stored.
RRN controls require stricter cell review for raw RRN exceptions.
RRN controls require pack version and policy digest on every decision.
RRN controls require deny-wins precedence.
RRN controls require no fallback to generic identifier policy.
RRN controls require bundle evidence for statutory basis mapping.
RRN controls require runtime validation tests before activation.
RRN controls require migration review for legacy RRN fields.
RRN controls require deletion path for prohibited legacy fields.
RRN controls require breach-classification linkage.
RRN controls require cross-border transfer denial by default.
RRN controls require processor due diligence if any processor touches derivative.
RRN controls require release checkpoint when policy changes.

## RRN Processing States

State `RRN-RAW-REJECTED` means raw value was submitted but denied.
State `RRN-RAW-TRANSIENT` means raw value exists only in a memory-bound validation window.
State `RRN-RAW-STATUTORY-HELD` means raw value is held under statutory basis.
State `RRN-DERIVED-HASHED` means irreversible derivative was created.
State `RRN-DERIVED-ROTATED` means derivative was rederived under new key context.
State `RRN-DERIVED-REVOKED` means derivative is no longer usable.
State `RRN-MIGRATION-QUARANTINED` means legacy RRN data is isolated.
State `RRN-MIGRATION-DELETED` means legacy prohibited raw data was deleted.
State `RRN-ACCESS-DENIED` means access was denied by Cedar.
State `RRN-ACCESS-APPROVED` means access was approved under purpose and role.
State `RRN-EXPORT-DENIED` means export was denied.
State `RRN-BREACH-CLOCK-STARTED` means leakage clock was started.
State `RRN-LEGAL-HOLD` means deletion is frozen under legal hold.
State `RRN-DISPOSAL-ELIGIBLE` means statutory need ended and deletion is queued.
State `RRN-DISPOSED` means raw or derivative disposal checkpoint completed.

## Statutory Basis Model

Basis code `KR-RRN-BASIS-PAYROLL-TAX` covers statutory payroll tax reporting.
Basis code `KR-RRN-BASIS-FOUR-MAJOR-INSURANCE` covers Korean statutory insurance administration.
Basis code `KR-RRN-BASIS-EMPLOYMENT-LAW` covers required employment administration.
Basis code `KR-RRN-BASIS-MEDICAL-CLAIM` covers legally required healthcare claim processing.
Basis code `KR-RRN-BASIS-COURT-ORDER` covers court or regulator order.
Basis code `KR-RRN-BASIS-PUBLIC-ADMIN` covers public administration mandate.
Basis code `KR-RRN-BASIS-IDENTITY-ALTERNATIVE-UNAVAILABLE` is not sufficient by itself.
Basis code `KR-RRN-BASIS-CONSENT` is invalid by itself.
Basis code `KR-RRN-BASIS-CONVENIENCE` is invalid.
Basis code `KR-RRN-BASIS-ANALYTICS` is invalid.
Basis code `KR-RRN-BASIS-SUPPORT` is invalid.
Basis code `KR-RRN-BASIS-TESTING` is invalid for real RRN.
Every valid basis must cite statute, decree, rule, or authority.
Every valid basis must name service purpose.
Every valid basis must name retention period.
Every valid basis must name approved field surface.
Every valid basis must name reviewer.
Every valid basis must name expiration or review date.
Every valid basis must name allowed processors.
Every valid basis must name deletion checkpoint.
Every valid basis must be auditable.
Every invalid basis must trigger denial.

## Alternative Identifiers

CI means connecting information token used in Korean identity ecosystems.
DI means duplication information token used in Korean identity ecosystems.
CI/DI may support identity assurance without raw RRN collection.
CI/DI must still be treated as personal information.
CI/DI must be tenant-scoped.
CI/DI must not be displayed to end users as account identifiers.
CI/DI must not be used as public URL components.
CI/DI must not be logged in raw form.
CI/DI must not be exported without transfer basis.
CI/DI must record identity provider code.
CI/DI must record assurance event ID.
CI/DI must record capture purpose.
CI/DI must record validity state.
CI/DI must record revocation state.
CI/DI must be tokenized where possible.
CI/DI must not be used to recreate RRN.
CI/DI is preferred for age verification when legally sufficient.
CI/DI is preferred for duplicate-account detection when legally sufficient.
CI/DI is preferred for patient portal identity when medically sufficient.
CI/DI is preferred for payroll onboarding only where statutory RRN is not required.
Alternative employee number is preferred for internal HR display.
Alternative patient number is preferred for hospital display.
Alternative customer number is preferred for support.
Alternative audit subject token is preferred for audit payloads.

## Irreversible Hashing Requirements

Hashing must be irreversible.
Hashing must use keyed derivation.
Hashing must use tenant-scoped context.
Hashing must use purpose-scoped context.
Hashing must use versioned key material.
Hashing must use approved cryptographic primitive.
Hashing must not use unsalted SHA-256 of RRN.
Hashing must not use reversible encryption as the ordinary derivative.
Hashing must not permit dictionary reconstruction.
Hashing must not share key context across tenants.
Hashing must not share key context across incompatible purposes.
Hashing must not place raw RRN in audit event.
Hashing must not place raw RRN in exception trace.
Hashing must not place raw RRN in job queue.
Hashing must not place raw RRN in dead-letter queue.
Hashing must not place raw RRN in metrics label.
Hashing must not place raw RRN in data warehouse.
Hashing must record key version.
Hashing must record derivation purpose.
Hashing must record statutory basis ID.
Hashing must record derivation timestamp.
Hashing must record hash algorithm profile.
Hashing must record rotation state.
Hashing must support key rotation.
Hashing must support derivative revocation.
Hashing must support deletion after statutory need ends.
Hashing must support breach impact classification.
Hashing must generate `KrRrnHashDerived`.

## Raw RRN Exception Handling

Raw RRN exception requires statutory basis.
Raw RRN exception requires legal reviewer.
Raw RRN exception requires security reviewer.
Raw RRN exception requires retention period.
Raw RRN exception requires field-level encryption if storage is mandatory.
Raw RRN exception requires KR identity cell placement.
Raw RRN exception requires break-glass access workflow.
Raw RRN exception requires dual-control approval where feasible.
Raw RRN exception requires access-purpose capture.
Raw RRN exception requires access event emission.
Raw RRN exception requires deletion checkpoint.
Raw RRN exception requires breach clock integration.
Raw RRN exception requires backup deletion plan.
Raw RRN exception requires non-production prohibition.
Raw RRN exception requires support tooling block.
Raw RRN exception requires search indexing block.
Raw RRN exception requires analytics block.
Raw RRN exception requires export block.
Raw RRN exception requires processor review.
Raw RRN exception expires unless renewed.

## Activated Cedar Policies

`pack-kr-pack-1-rrn-collection-deny-default` denies raw RRN collection by default.
`pack-kr-pack-1-rrn-statutory-basis` permits RRN processing only with basis evidence.
`pack-kr-pack-1-rrn-hash-only` requires irreversible derivative for persistent matching.
`pack-kr-pack-1-ci-di-preferred` prefers non-RRN alternative identifiers.
`pack-kr-pack-1-cell-kr-residency` pins RRN derivatives to KR cell.
`pack-kr-pack-1-kisa-bio-cell` applies when biometric identity and RRN-adjacent evidence combine.
`pack-kr-pack-1-cross-border-transfer-deny-default` blocks export.
`pack-kr-pack-1-pipa-breach-reporting-window` starts breach clocks for RRN leakage.
`pack-kr-pack-1-pii-emission-scrub` blocks RRN in audit payloads.
`pack-kr-pack-1-audit-tenant-context` requires tenant context.
`pack-kr-pack-1-audit-jurisdiction-code` requires KR jurisdiction code.
`pack-kr-pack-1-retention-legal-hold` blocks deletion when legal hold exists.
`pack-kr-pack-1-processor-due-diligence` requires processor review.
`pack-kr-pack-1-pack-precedence-deny-wins` prevents generic identifier policy override.
`pack-kr-pack-1-pipa-purpose-consent` still applies to surrounding personal information.
`pack-kr-pack-1-pipa-sensitive-consent` applies when RRN appears with sensitive data.
`pack-kr-pack-1-localized-notice-required` requires Korean notice for RRN-adjacent workflow.

## Data Model Deltas

Add `identity.kr_rrn_processing_state`.
Add `identity.kr_rrn_present_flag`.
Add `identity.kr_rrn_raw_storage_prohibited_flag`.
Add `identity.kr_rrn_statutory_basis_id`.
Add `identity.kr_rrn_statutory_basis_code`.
Add `identity.kr_rrn_statutory_basis_authority`.
Add `identity.kr_rrn_basis_reviewer`.
Add `identity.kr_rrn_basis_reviewed_at`.
Add `identity.kr_rrn_basis_expires_at`.
Add `identity.kr_rrn_retention_period`.
Add `identity.kr_rrn_disposal_due_at`.
Add `identity.kr_rrn_disposal_checkpoint_id`.
Add `identity.kr_rrn_hash_digest`.
Add `identity.kr_rrn_hash_key_version`.
Add `identity.kr_rrn_hash_algorithm_profile`.
Add `identity.kr_rrn_hash_purpose_context`.
Add `identity.kr_rrn_hash_tenant_context`.
Add `identity.kr_rrn_hash_created_at`.
Add `identity.kr_rrn_hash_rotated_at`.
Add `identity.kr_rrn_hash_revoked_at`.
Add `identity.kr_rrn_access_approval_id`.
Add `identity.kr_rrn_last_access_audit_id`.
Add `identity.kr_ci_token`.
Add `identity.kr_di_token`.
Add `identity.kr_ci_di_provider_code`.
Add `identity.kr_identity_assurance_event_id`.
Add `identity.kr_identity_assurance_level`.
Add `identity.kr_identity_token_status`.
Add `identity.kr_identity_token_revoked_at`.
Add `identity.kr_display_identifier`.
Add `identity.kr_support_identifier`.
Add `identity.kr_audit_subject_token`.
Add `identity.kr_legacy_rrn_quarantine_id`.
Add `identity.kr_legacy_rrn_cleanup_state`.
Add `identity.kr_rrn_breach_clock_id`.
Add `identity.kr_rrn_legal_hold_state`.
Transform raw RRN input into transient validation envelope.
Transform transient RRN into irreversible derivative.
Transform legacy raw RRN into quarantine record.
Transform display identity into non-RRN identifier.
Transform audit subject identity into scrubbed token.
Transform CI/DI linkage into identity assurance record.
Transform statutory basis into reviewable evidence object.
Transform RRN access into privileged event.
Transform RRN deletion into checkpointed disposal event.

## API Contract Deltas

`POST /kr/identity/rrn/validate` validates raw RRN transiently.
`POST /kr/identity/rrn/validate` requires statutory basis intent or denial-mode flag.
`POST /kr/identity/rrn/validate` returns validity result only.
`POST /kr/identity/rrn/validate` returns no raw RRN.
`POST /kr/identity/rrn/statutory-basis` records basis evidence.
`GET /kr/identity/rrn/statutory-basis/{basis_id}` returns basis status.
`POST /kr/identity/rrn/hash` creates irreversible derivative.
`POST /kr/identity/rrn/hash` requires basis ID.
`POST /kr/identity/rrn/hash` requires tenant context.
`POST /kr/identity/rrn/hash` returns digest fingerprint and key version only.
`POST /kr/identity/rrn/access-request` requests exceptional raw access where legally held.
`POST /kr/identity/rrn/access-request/{id}/approve` approves exceptional access.
`POST /kr/identity/rrn/access-request/{id}/deny` denies exceptional access.
`POST /kr/identity/rrn/dispose` disposes raw or derivative when eligible.
`POST /kr/identity/rrn/migration/quarantine` quarantines legacy RRN field.
`POST /kr/identity/rrn/migration/delete` deletes prohibited legacy RRN field.
`POST /kr/identity/ci-di/link` links alternative identity tokens.
`POST /kr/identity/ci-di/revoke` revokes alternative identity tokens.
`GET /kr/identity/{subject_id}/display-id` returns non-RRN display identifier.
`GET /kr/identity/{subject_id}/audit-token` returns scrubbed audit subject token.
Every RRN API requires `tenant_id`.
Every scoped RRN API requires `sub_scope_path`.
Every state-changing RRN API returns `audit_id`.
Every RRN API returns `jurisdiction_code=KR`.
Every RRN API denies raw RRN in response bodies.
Every RRN API denies raw RRN in error messages.
Every RRN API denies raw RRN in logs.
Every RRN API returns `cedar_policy_ids`.

## Audit Event Additions

`KrRrnCollectionAttempted` records attempted collection surface.
`KrRrnCollectionDenied` records denial reason and policy ID.
`KrRrnStatutoryBasisSubmitted` records basis code and authority reference.
`KrRrnStatutoryBasisAccepted` records reviewer and scope.
`KrRrnStatutoryBasisDenied` records denial reason.
`KrRrnValidationPerformed` records transient validation without raw value.
`KrRrnHashDerived` records derivative key version and purpose context.
`KrRrnHashRotated` records old and new key versions through fingerprints only.
`KrRrnHashRevoked` records derivative revocation.
`KrRrnRawStorageExceptionApproved` records raw retention exception.
`KrRrnRawAccessRequested` records access purpose.
`KrRrnRawAccessApproved` records approver and expiration.
`KrRrnRawAccessDenied` records denial reason.
`KrRrnRawAccessed` records privileged access event.
`KrRrnDisplaySuppressed` records blocked display attempt.
`KrRrnExportDenied` records blocked export attempt.
`KrRrnLegacyQuarantined` records legacy field quarantine.
`KrRrnLegacyDeleted` records legacy field deletion checkpoint.
`KrRrnDisposalQueued` records disposal eligibility.
`KrRrnDisposed` records disposal completion.
`KrCiDiLinked` records alternative identity linkage.
`KrCiDiRevoked` records alternative identity revocation.
`KrRrnBreachClockStarted` records incident clock start.
Every event carries `tenant_id`.
Every event carries `sub_scope_path` where scoped.
Every event carries `event_id`.
Every event carries `trace_id`.
Every event carries `span_id`.
Every event carries `audit_id`.
Every event carries `schema_version`.
Every event carries `source_microservice`.
Every event carries `cell_id`.
Every event carries `jurisdiction_code=KR`.
Every event payload is PII-scrubbed.
No event payload may contain raw RRN.

## Failure Modes specific to KR enforcement

Failure mode `KR-RRN-FM-001`: raw RRN accepted without statutory basis.
Failure mode `KR-RRN-FM-002`: RRN collected through generic consent.
Failure mode `KR-RRN-FM-003`: RRN stored as user identifier.
Failure mode `KR-RRN-FM-004`: RRN displayed in UI.
Failure mode `KR-RRN-FM-005`: RRN written to logs.
Failure mode `KR-RRN-FM-006`: RRN written to audit payload.
Failure mode `KR-RRN-FM-007`: RRN written to support ticket.
Failure mode `KR-RRN-FM-008`: RRN indexed in search.
Failure mode `KR-RRN-FM-009`: RRN exported to BI.
Failure mode `KR-RRN-FM-010`: RRN used in model training.
Failure mode `KR-RRN-FM-011`: unsalted hash used as derivative.
Failure mode `KR-RRN-FM-012`: reversible encryption used as ordinary derivative.
Failure mode `KR-RRN-FM-013`: hash context shared across tenants.
Failure mode `KR-RRN-FM-014`: hash context shared across incompatible purposes.
Failure mode `KR-RRN-FM-015`: hash key version missing.
Failure mode `KR-RRN-FM-016`: statutory basis lacks authority reference.
Failure mode `KR-RRN-FM-017`: statutory basis lacks reviewer.
Failure mode `KR-RRN-FM-018`: statutory basis lacks retention period.
Failure mode `KR-RRN-FM-019`: statutory basis expired.
Failure mode `KR-RRN-FM-020`: raw storage exception lacks dual approval.
Failure mode `KR-RRN-FM-021`: raw storage exception lacks deletion checkpoint.
Failure mode `KR-RRN-FM-022`: CI token displayed as public identifier.
Failure mode `KR-RRN-FM-023`: DI token crosses tenant boundary.
Failure mode `KR-RRN-FM-024`: alternative ID provider code missing.
Failure mode `KR-RRN-FM-025`: legacy RRN field not quarantined.
Failure mode `KR-RRN-FM-026`: legacy RRN cleanup deletes legal evidence without review.
Failure mode `KR-RRN-FM-027`: RRN derivative exported without assessment.
Failure mode `KR-RRN-FM-028`: raw RRN exported.
Failure mode `KR-RRN-FM-029`: RRN breach clock not started.
Failure mode `KR-RRN-FM-030`: RRN state change omits audit ID.

## Worked Examples

### Scenario 1: Payroll RRN Derivative

Payroll service needs a statutory RRN workflow.
The service submits basis code `KR-RRN-BASIS-PAYROLL-TAX`.
The basis cites applicable payroll authority.
The reviewer approves the basis.
The RRN validation endpoint receives raw RRN transiently.
The endpoint validates format and checksum.
The raw value is not persisted.
The hash endpoint derives irreversible tenant-scoped digest.
The response returns digest fingerprint and key version.
The audit stream emits `KrRrnStatutoryBasisAccepted`.
The audit stream emits `KrRrnHashDerived`.
The payroll record stores derivative only.
Search index receives non-RRN display identifier.

### Scenario 2: Marketing Form RRN Attempt

Marketing form includes an RRN field by mistake.
The form submits raw RRN with generic marketing consent.
The RRN collection policy denies the request.
The API returns `KR-RRN-FM-002`.
The audit stream emits `KrRrnCollectionDenied`.
The raw value is dropped at boundary.
No support ticket includes the value.
No telemetry includes the value.
The service owner receives remediation ticket.
The consent record remains invalid for RRN.

### Scenario 3: CI/DI Age Verification

Community service needs adult verification.
The age gate uses an approved identity provider.
The provider returns age-band and CI/DI token.
The service records provider code.
The service records assurance event ID.
The service receives adult age-band.
The service does not receive RRN.
The audit stream emits `KrCiDiLinked`.
The youth access policy permits adult-only board access.
The CI/DI token remains hidden from user-facing identifiers.

### Scenario 4: Legacy Raw RRN Cleanup

A migration scanner finds legacy raw RRN field.
The migration endpoint quarantines the field.
The audit stream emits `KrRrnLegacyQuarantined`.
Legal review determines no statutory retention need.
Deletion checkpoint is created.
The raw field is deleted from primary store.
Backup deletion plan is scheduled.
The audit stream emits `KrRrnLegacyDeleted`.
Derivative is re-created only if statutory basis exists.
The service cannot search or display legacy RRN.

### Scenario 5: RRN Leakage Incident

Security detects possible RRN leakage.
The incident classifier detects `PI_KR_RRN`.
The breach clock starts immediately.
The audit stream emits `KrRrnBreachClockStarted`.
PIPA reporting workflow starts.
KISA/PIPC notification tasks are created where criteria apply.
The incident evidence remains in KR incident cell.
Raw RRN samples are scrubbed from incident tickets.
Containment blocks export and support access.
Post-incident disposal review runs after legal hold is resolved.

## Cross-References

Pack overview: `packs/kr-localization/README.md`.
Regulatory coverage: `packs/kr-localization/regulatory-coverage.md`.
Data residency: `packs/kr-localization/data-residency.md`.
Consent management: `packs/kr-localization/consent-management.md`.
Incident response: `packs/kr-localization/cybersecurity-and-incident-response.md`.
ADR-0064 localization pack architecture: `docs/decisions/ADR-0709-general-live-apex.md`.
ADR-0244 tenant scoping: `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
ADR-0251 compliance pack mechanics: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`.
ADR-0263 audit event emission contract: `docs/decisions/ADR-0706-observability-live-apex.md`.
KR pack manifest seed: `docs/localization-packs/kr/pack.yaml`.
Official law source: `https://www.law.go.kr/`.
Official PIPC source: `https://www.pipc.go.kr/`.
Official KISA source: `https://www.kisa.or.kr/`.

## RRN Requirement Register

`KR-RRN-REQ-001` RRN collection is denied by default.
`KR-RRN-REQ-002` RRN collection requires statutory basis.
`KR-RRN-REQ-003` RRN collection cannot rely on consent alone.
`KR-RRN-REQ-004` RRN validation must be transient by default.
`KR-RRN-REQ-005` RRN validation response must exclude raw value.
`KR-RRN-REQ-006` RRN derivative must be irreversible.
`KR-RRN-REQ-007` RRN derivative must use keyed derivation.
`KR-RRN-REQ-008` RRN derivative must use tenant context.
`KR-RRN-REQ-009` RRN derivative must use purpose context.
`KR-RRN-REQ-010` RRN derivative must record key version.
`KR-RRN-REQ-011` RRN derivative must record algorithm profile.
`KR-RRN-REQ-012` RRN derivative must support rotation.
`KR-RRN-REQ-013` RRN derivative must support revocation.
`KR-RRN-REQ-014` RRN derivative must support disposal.
`KR-RRN-REQ-015` unsalted hashing is prohibited.
`KR-RRN-REQ-016` reversible ordinary derivative is prohibited.
`KR-RRN-REQ-017` raw RRN logs are prohibited.
`KR-RRN-REQ-018` raw RRN metrics are prohibited.
`KR-RRN-REQ-019` raw RRN audit payloads are prohibited.
`KR-RRN-REQ-020` raw RRN support tickets are prohibited.
`KR-RRN-REQ-021` raw RRN search indexes are prohibited.
`KR-RRN-REQ-022` raw RRN BI exports are prohibited.
`KR-RRN-REQ-023` raw RRN training datasets are prohibited.
`KR-RRN-REQ-024` raw RRN URL parameters are prohibited.
`KR-RRN-REQ-025` raw RRN display fields are prohibited.
`KR-RRN-REQ-026` raw RRN exception requires statutory basis.
`KR-RRN-REQ-027` raw RRN exception requires legal reviewer.
`KR-RRN-REQ-028` raw RRN exception requires security reviewer.
`KR-RRN-REQ-029` raw RRN exception requires retention period.
`KR-RRN-REQ-030` raw RRN exception requires deletion checkpoint.
`KR-RRN-REQ-031` raw RRN exception requires field encryption.
`KR-RRN-REQ-032` raw RRN exception requires KR identity cell.
`KR-RRN-REQ-033` raw RRN exception requires privileged access workflow.
`KR-RRN-REQ-034` raw RRN exception requires breach integration.
`KR-RRN-REQ-035` raw RRN exception requires backup deletion plan.
`KR-RRN-REQ-036` raw RRN exception expires by default.
`KR-RRN-REQ-037` statutory basis must cite authority.
`KR-RRN-REQ-038` statutory basis must name purpose.
`KR-RRN-REQ-039` statutory basis must name allowed fields.
`KR-RRN-REQ-040` statutory basis must name retention.
`KR-RRN-REQ-041` statutory basis must name reviewer.
`KR-RRN-REQ-042` statutory basis must name processors.
`KR-RRN-REQ-043` statutory basis must name expiration or review date.
`KR-RRN-REQ-044` statutory basis must be auditable.
`KR-RRN-REQ-045` invalid basis must deny collection.
`KR-RRN-REQ-046` convenience basis is invalid.
`KR-RRN-REQ-047` analytics basis is invalid.
`KR-RRN-REQ-048` support basis is invalid.
`KR-RRN-REQ-049` generic consent basis is invalid.
`KR-RRN-REQ-050` testing basis with real RRN is invalid.
`KR-RRN-REQ-051` CI token must be treated as personal information.
`KR-RRN-REQ-052` DI token must be treated as personal information.
`KR-RRN-REQ-053` CI/DI must be tenant-scoped.
`KR-RRN-REQ-054` CI/DI must not be public identifier.
`KR-RRN-REQ-055` CI/DI must not be logged raw.
`KR-RRN-REQ-056` CI/DI must record provider.
`KR-RRN-REQ-057` CI/DI must record assurance event.
`KR-RRN-REQ-058` CI/DI must support revocation.
`KR-RRN-REQ-059` CI/DI must not reconstruct RRN.
`KR-RRN-REQ-060` CI/DI should be preferred for age gates.
`KR-RRN-REQ-061` employee display ID must not be RRN.
`KR-RRN-REQ-062` patient display ID must not be RRN.
`KR-RRN-REQ-063` customer display ID must not be RRN.
`KR-RRN-REQ-064` support display ID must not be RRN.
`KR-RRN-REQ-065` audit subject token must not be RRN.
`KR-RRN-REQ-066` RRN API requires tenant ID.
`KR-RRN-REQ-067` RRN API requires scoped context where applicable.
`KR-RRN-REQ-068` RRN API returns audit ID for state changes.
`KR-RRN-REQ-069` RRN API returns jurisdiction code KR.
`KR-RRN-REQ-070` RRN API returns policy IDs.
`KR-RRN-REQ-071` RRN API returns failure mode on denial.
`KR-RRN-REQ-072` RRN API must scrub error body.
`KR-RRN-REQ-073` RRN API must scrub validation failures.
`KR-RRN-REQ-074` RRN API must scrub tracing spans.
`KR-RRN-REQ-075` RRN access request must name purpose.
`KR-RRN-REQ-076` RRN access request must name role.
`KR-RRN-REQ-077` RRN access request must name expiration.
`KR-RRN-REQ-078` RRN access approval must name approver.
`KR-RRN-REQ-079` RRN access must emit privileged event.
`KR-RRN-REQ-080` RRN access denial must emit denial event.
`KR-RRN-REQ-081` RRN disposal must check legal hold.
`KR-RRN-REQ-082` RRN disposal must check statutory retention.
`KR-RRN-REQ-083` RRN disposal must check incident hold.
`KR-RRN-REQ-084` RRN disposal must record checkpoint.
`KR-RRN-REQ-085` RRN disposal must include backups where feasible.
`KR-RRN-REQ-086` RRN migration must quarantine legacy fields.
`KR-RRN-REQ-087` RRN migration must inventory indexes.
`KR-RRN-REQ-088` RRN migration must inventory logs.
`KR-RRN-REQ-089` RRN migration must inventory data warehouse.
`KR-RRN-REQ-090` RRN migration must inventory tickets.
`KR-RRN-REQ-091` RRN migration must inventory backups.
`KR-RRN-REQ-092` RRN migration must delete prohibited copies.
`KR-RRN-REQ-093` RRN migration must preserve legal evidence.
`KR-RRN-REQ-094` RRN migration must emit checkpoint.
`KR-RRN-REQ-095` RRN breach classification must start PIPA clock.
`KR-RRN-REQ-096` RRN breach classification must assess KISA path.
`KR-RRN-REQ-097` RRN breach classification must assess subject notification.
`KR-RRN-REQ-098` RRN breach classification must preserve evidence in KR.
`KR-RRN-REQ-099` RRN breach evidence must be scrubbed in tickets.
`KR-RRN-REQ-100` RRN breach evidence must block non-KR export.
`KR-RRN-REQ-101` RRN policy must deny before service persistence.
`KR-RRN-REQ-102` RRN policy must deny before queue enqueue.
`KR-RRN-REQ-103` RRN policy must deny before file upload.
`KR-RRN-REQ-104` RRN policy must deny before email attachment.
`KR-RRN-REQ-105` RRN policy must deny before spreadsheet export.
`KR-RRN-REQ-106` RRN policy must deny before support transcript save.
`KR-RRN-REQ-107` RRN policy must deny before OCR extraction persistence.
`KR-RRN-REQ-108` RRN policy must deny before image metadata persistence.
`KR-RRN-REQ-109` RRN policy must deny before document AI ingestion.
`KR-RRN-REQ-110` RRN policy must deny before search indexing.
`KR-RRN-REQ-111` RRN policy must be tested per service.
`KR-RRN-REQ-112` RRN policy must be tested for logging.
`KR-RRN-REQ-113` RRN policy must be tested for audit payloads.
`KR-RRN-REQ-114` RRN policy must be tested for export.
`KR-RRN-REQ-115` RRN policy must be tested for migration.
`KR-RRN-REQ-116` RRN policy must be tested for breach classification.
`KR-RRN-REQ-117` RRN policy must be tested for consent-only denial.
`KR-RRN-REQ-118` RRN policy must be tested for statutory basis approval.
`KR-RRN-REQ-119` RRN policy must be tested for alternative ID flow.
`KR-RRN-REQ-120` RRN policy must be tested for pack deactivation denial.

## Checkpoint

This file is scoped to `/packs/kr-localization/`.
It does not edit ADRs.
It does not edit microservices.
It does not edit other packs.
It must be line-count verified with the rest of KR-PACK-1.
It must be lifecycle-verified with retired VCS ratchet after all six docs exist.
