---
doc_class: CrossMicroserviceIntegrationTest
scenario_id: CMIT-004-dual-tenant-document-export
microservices_under_test:
  - drive
  - identity
  - governance
  - audit-chain
status: draft-canonical
date: 2026-05-20
owner: codex-cross-msvc-integration-tests-w1
related_oyatie_adrs:
  - ADR-0113-vcs-orchestrator-end-to-end
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0263-observability-emission-contract
  - ADR-0276-backup-portability-gdpr-art-20
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0312-court-warrant-scoped-piercing
---

# Dual-Tenant Document Export

## Scenario Description

Chris Volkov, a designer newly hired into `tenant-northstar-studios-us`, intentionally exports a portfolio file from his personal tenant `personal-chris-volkov` into his employer work Drive; ADR-0311 requires the same passkey to bridge the contexts while proving that the employer cannot pull personal files, cannot list Chris's personal Drive, and receives only the explicitly exported artifact.

## Pre-conditions

### Named tenant fixtures

- Personal tenant: `personal-chris-volkov`.
- Work tenant: `tenant-northstar-studios-us`.
- Human principal: `principal-chris-volkov`.
- HR admin principal: `principal-priya-menon-hr`.
- Work manager principal: `principal-sam-lewis-design`.
- Shared passkey credential: `passkey-chris-volkov-2026-05`.
- Personal source file: `drive-file-personal-portfolio-2026`.
- Work destination folder: `drive-folder-northstar-onboarding-design`.
- Export request id: `export-chris-portfolio-to-northstar-001`.
- Work copy id: `drive-file-work-portfolio-copy-001`.
- Personal Drive space: `drive-space-personal-chris`.
- Work Drive space: `drive-space-northstar-design`.
- Trace id: `trace-cmit-004-dual-tenant-export`.
- Idempotency key: `idem-cmit-004-export`.

### Named Cedar permits

- `permit-personal-owner-export-own-document`.
- `permit-work-tenant-receive-user-export`.
- `permit-identity-tenant-context-switch-self`.
- `permit-governance-export-consent-record`.
- `permit-audit-seal-personal-export`.
- `permit-audit-seal-work-import`.
- `forbid-work-admin-pull-personal-drive`.
- `forbid-work-admin-list-personal-drive`.
- `forbid-cross-tenant-export-without-user-consent`.

### Named pack activations

- `pack-ADR-0311-work-personal-boundary`.
- `pack-GDPR-portability`.
- `pack-US-state-privacy-baseline`.
- `pack-SOC2-Type-II-baseline`.
- `pack-employee-onboarding-consent`.
- `pack-document-dlp-baseline`.

### Starting state checks

- Chris has one passkey and two tenant memberships.
- Active session starts in personal tenant context.
- Personal source file exists and is owned by `personal-chris-volkov`.
- Work destination folder exists and is owned by `tenant-northstar-studios-us`.
- HR admin has work-tenant audit permissions only.
- HR admin has no personal-tenant grants for Chris.
- Governance export consent ledger is empty for `export-chris-portfolio-to-northstar-001`.
- Audit streams for both tenants exist and are empty for this trace.

## Test Steps

1. Authenticate Chris using the shared passkey.
   - API call: `POST /identity/v1/sessions/passkey`.
   - Expected response: `201 Created` with `principal_id="principal-chris-volkov"` and two tenant choices.
   - Assertion: no tenant context is active until Chris selects one.

2. Select personal tenant context.
   - API call: `POST /identity/v1/sessions/{session_id}/tenant-context` with `tenant_id="personal-chris-volkov"`.
   - Expected response: `200 OK` with `X-Oya-Tenant-Context: personal-chris-volkov`.
   - Assertion: token audience is personal and cannot write work Drive yet.

3. Read the personal portfolio metadata.
   - API call: `GET /drive/v1/files/drive-file-personal-portfolio-2026`.
   - Expected response: `200 OK` with `tenant_ownership_class="PERSONAL_TENANT"`.
   - Assertion: file content hash is returned only to the personal owner.

4. HR admin attempts to pull the personal file.
   - API call: `GET /drive/v1/files/drive-file-personal-portfolio-2026` as `principal-priya-menon-hr`.
   - Expected response: `403 Forbidden` with `error.code="WORK_ADMIN_PERSONAL_TENANT_DENY"`.
   - Assertion: denial cites `forbid-work-admin-pull-personal-drive`.

5. HR admin attempts to list Chris's personal Drive.
   - API call: `GET /drive/v1/spaces/drive-space-personal-chris/files` as `principal-priya-menon-hr`.
   - Expected response: `403 Forbidden` with `error.code="WORK_ADMIN_PERSONAL_LIST_DENY"`.
   - Assertion: response reveals no file count, file names, or existence hints.

6. Start an explicit export consent ceremony.
   - API call: `POST /governance/v1/dual-tenant-exports`.
   - Expected response: `201 Created` with `export_request_id="export-chris-portfolio-to-northstar-001"` and `state="AWAITING_PERSONAL_OWNER_CONSENT"`.
   - Assertion: export request names source personal tenant and destination work tenant separately.

7. Render export disclosure.
   - API call: `GET /governance/v1/dual-tenant-exports/export-chris-portfolio-to-northstar-001/disclosure`.
   - Expected response: `200 OK` with destination tenant, destination folder, file name, hash, and revocation note.
   - Assertion: disclosure includes ADR-0311 boundary language and does not ask for blanket personal Drive access.

8. Chris consents to export exactly one file.
   - API call: `POST /governance/v1/dual-tenant-exports/export-chris-portfolio-to-northstar-001:consent`.
   - Expected response: `200 OK` with `state="CONSENTED"` and `cedar_decision_id="cedar-cmit-004-export-consent"`.
   - Assertion: consent scope contains one source file and one destination folder.

9. Seal personal-side consent.
   - API call: `POST /audit-chain/v1/streams/personal-chris-volkov.drive/events`.
   - Expected response: `201 Created` with `event_class="TenantBoundaryPersonalExportConsent"`.
   - Assertion: event is visible to Chris's personal tenant and not to the work tenant admin by default.

10. Switch to work tenant context for receive verification.
    - API call: `POST /identity/v1/sessions/{session_id}/tenant-context` with `tenant_id="tenant-northstar-studios-us"`.
    - Expected response: `200 OK` with `X-Oya-Tenant-Context: tenant-northstar-studios-us`.
    - Assertion: token cannot read the personal source file without export grant.

11. Governance evaluates receive-side permit.
    - API call: `POST /governance/v1/policy-decisions:check` with action `drive.export.receive`.
    - Expected response: `200 OK` with `decision="ALLOW"` and `cedar_decision_id="cedar-cmit-004-work-receive"`.
    - Assertion: decision requires prior personal consent proof.

12. Execute the Drive export copy.
    - API call: `POST /drive/v1/dual-tenant-exports/export-chris-portfolio-to-northstar-001:copy`.
    - Expected response: `201 Created` with `work_file_id="drive-file-work-portfolio-copy-001"`.
    - Assertion: work copy has new object id, destination tenant owner, and provenance link to source hash.

13. Verify personal source remains unchanged.
    - API call: `GET /drive/v1/files/drive-file-personal-portfolio-2026`.
    - Expected response: `200 OK` with original version and original hash.
    - Assertion: export is copy semantics, not ownership transfer.

14. Seal work-side import.
    - API call: `POST /audit-chain/v1/streams/tenant-northstar-studios-us.drive/events`.
    - Expected response: `201 Created` with `event_class="TenantBoundaryWorkImportFromPersonal"`.
    - Assertion: work event includes source hash and consent proof hash, not the personal tenant file path.

15. Work manager reads the imported copy.
    - API call: `GET /drive/v1/files/drive-file-work-portfolio-copy-001` as `principal-sam-lewis-design`.
    - Expected response: `200 OK` with work-owned copy metadata.
    - Assertion: manager can read only the imported copy, not the personal source.

16. Work manager attempts provenance source read.
    - API call: `GET /drive/v1/files/drive-file-personal-portfolio-2026?via=provenance`.
    - Expected response: `403 Forbidden` with `error.code="PROVENANCE_NOT_CONTENT_GRANT"`.
    - Assertion: provenance link does not become a content access grant.

17. Re-submit the copy request idempotently.
    - API call: `POST /drive/v1/dual-tenant-exports/export-chris-portfolio-to-northstar-001:copy` with same idempotency key.
    - Expected response: `200 OK` with the same `work_file_id`.
    - Assertion: no second work copy and no duplicate import audit event.

18. Revoke future export consent.
    - API call: `POST /governance/v1/dual-tenant-exports/export-chris-portfolio-to-northstar-001:revoke-future`.
    - Expected response: `200 OK` with `future_exports_allowed=false`.
    - Assertion: existing work copy remains work-owned, future reads of personal source are denied.

19. Attempt second file export without new consent.
    - API call: `POST /drive/v1/dual-tenant-exports` targeting `drive-file-personal-tax-return-2025`.
    - Expected response: `403 Forbidden` with `error.code="MISSING_USER_CONSENT"`.
    - Assertion: previous consent cannot be generalized to other files.

20. Query personal audit view.
    - API call: `GET /audit-chain/v1/streams/personal-chris-volkov.drive/events?trace_id=trace-cmit-004-dual-tenant-export`.
    - Expected response: `200 OK` with consent and export-read events.
    - Assertion: personal audit view shows the exact file exported and destination tenant.

21. Query work audit view.
    - API call: `GET /audit-chain/v1/streams/tenant-northstar-studios-us.drive/events?trace_id=trace-cmit-004-dual-tenant-export`.
    - Expected response: `200 OK` with import event and work-copy read events.
    - Assertion: work audit view shows consent proof hash but not personal Drive listing.

22. Verify boundary summary.
    - API call: `GET /governance/v1/dual-tenant-exports/export-chris-portfolio-to-northstar-001/boundary-summary`.
    - Expected response: `200 OK` with `personal_source_preserved=true`, `work_copy_created=true`, and `blanket_access_granted=false`.
    - Assertion: summary proves ADR-0311 personal/work boundary held for the full flow.

## Test Data Fixtures

### Fixture `DualTenantIdentityFixture`

```json
{
  "principal_id": "principal-chris-volkov",
  "passkey_credential_id": "passkey-chris-volkov-2026-05",
  "tenant_memberships": [
    {
      "tenant_id": "personal-chris-volkov",
      "audience_type": "B2C_CONSUMER",
      "role": "PERSONAL_OWNER"
    },
    {
      "tenant_id": "tenant-northstar-studios-us",
      "audience_type": "B2B_EMPLOYEE",
      "role": "DESIGNER"
    }
  ]
}
```

### Fixture `PersonalSourceFileFixture`

```json
{
  "file_id": "drive-file-personal-portfolio-2026",
  "space_id": "drive-space-personal-chris",
  "owner_tenant_id": "personal-chris-volkov",
  "tenant_ownership_class": "PERSONAL_TENANT",
  "name": "portfolio-2026.pdf",
  "sha256": "sha256:personal-portfolio-2026",
  "version": 7,
  "classification": "PERSONAL_CREATIVE_WORK"
}
```

### Fixture `WorkDestinationFixture`

```yaml
folder_id: drive-folder-northstar-onboarding-design
space_id: drive-space-northstar-design
owner_tenant_id: tenant-northstar-studios-us
tenant_ownership_class: WORK_TENANT
allowed_receivers:
  - principal-chris-volkov
  - principal-sam-lewis-design
retention_policy: employee-portfolio-review-2y
```

### Fixture `ExportConsentFixture`

```json
{
  "export_request_id": "export-chris-portfolio-to-northstar-001",
  "source_tenant_id": "personal-chris-volkov",
  "destination_tenant_id": "tenant-northstar-studios-us",
  "source_file_id": "drive-file-personal-portfolio-2026",
  "destination_folder_id": "drive-folder-northstar-onboarding-design",
  "consented_by": "principal-chris-volkov",
  "consent_scope": "ONE_FILE_ONE_COPY",
  "expires_at": "2026-05-20T14:30:00Z"
}
```

### Fixture `WorkCopyFixture`

```json
{
  "work_file_id": "drive-file-work-portfolio-copy-001",
  "owner_tenant_id": "tenant-northstar-studios-us",
  "tenant_ownership_class": "WORK_TENANT",
  "source_hash": "sha256:personal-portfolio-2026",
  "source_file_id_redacted": true,
  "provenance_link_type": "HASH_AND_CONSENT_PROOF_ONLY",
  "retention_policy": "employee-portfolio-review-2y"
}
```

### Fixture `BoundaryAuditFixture`

```yaml
trace_id: trace-cmit-004-dual-tenant-export
personal_events:
  - TenantBoundaryPersonalExportConsent
  - PersonalDriveExportRead
  - PersonalExportFutureConsentRevoked
work_events:
  - TenantBoundaryWorkImportFromPersonal
  - WorkDriveCopyCreated
  - WorkDriveCopyRead
denial_events:
  - WorkAdminPersonalDriveReadDenied
  - WorkAdminPersonalDriveListDenied
  - ProvenanceSourceReadDenied
```

## Assertion Catalogue

### What passes

- `PASS-ID-001`: one passkey maps to personal and work memberships.
- `PASS-ID-002`: session has no active tenant before explicit selection.
- `PASS-ID-003`: tenant switch changes token context.
- `PASS-DRIVE-001`: personal file is marked `PERSONAL_TENANT`.
- `PASS-DRIVE-002`: work destination folder is marked `WORK_TENANT`.
- `PASS-GOV-001`: export request names source and destination tenants.
- `PASS-GOV-002`: consent scope is exactly one file and one destination.
- `PASS-GOV-003`: receive-side permit requires personal consent proof.
- `PASS-EXPORT-001`: work copy has a new object id.
- `PASS-EXPORT-002`: source file version and hash remain unchanged.
- `PASS-EXPORT-003`: provenance link does not grant source content access.
- `PASS-EXPORT-004`: idempotent retry returns same work copy.
- `PASS-BOUNDARY-001`: HR admin cannot pull personal file.
- `PASS-BOUNDARY-002`: HR admin cannot list personal Drive.
- `PASS-BOUNDARY-003`: work manager cannot read source through provenance.
- `PASS-AUDIT-001`: personal stream records consent.
- `PASS-AUDIT-002`: work stream records import.
- `PASS-AUDIT-003`: work stream sees proof hash, not personal listing.
- `PASS-REVOKE-001`: future consent revocation blocks additional file export.
- `PASS-SUMMARY-001`: boundary summary proves no blanket access.

### What fails

- `FAIL-ID-001`: active tenant defaults silently.
- `FAIL-DRIVE-001`: personal file classified as work-owned.
- `FAIL-GOV-001`: consent omits destination tenant.
- `FAIL-GOV-002`: consent grants folder-wide personal export.
- `FAIL-EXPORT-001`: export transfers ownership instead of copying.
- `FAIL-EXPORT-002`: work copy reuses personal file id.
- `FAIL-BOUNDARY-001`: work admin can pull personal source.
- `FAIL-BOUNDARY-002`: work admin can list personal Drive.
- `FAIL-BOUNDARY-003`: manager can follow provenance to source content.
- `FAIL-AUDIT-001`: personal consent not sealed.
- `FAIL-AUDIT-002`: work import not sealed.
- `FAIL-AUDIT-003`: work audit exposes personal file path.
- `FAIL-IDEMPOTENCY-001`: retry creates duplicate work copy.
- `FAIL-REVOKE-001`: future consent revocation is ignored.
- `FAIL-SUMMARY-001`: boundary summary claims blanket grant.

## Failure Mode Coverage

- `FM-DTB-001`: per-service tenant ownership drift.
- `FM-DTB-002`: Cedar permit over-scope grants work admin personal read.
- `FM-DTB-003`: UI/session confusion silently acts in wrong tenant.
- `FM-DTB-004`: export is implemented as ownership transfer.
- `FM-DTB-005`: work destination receives personal file path metadata.
- `FM-DTB-006`: consent ceremony creates blanket access.
- `FM-DTB-007`: prior consent reused for a different personal file.
- `FM-DTB-008`: HR principal enumerates personal Drive through list endpoint.
- `FM-DTB-009`: manager follows provenance link into source content.
- `FM-DTB-010`: audit event co-mingles personal and work streams.
- `FM-DTB-011`: personal source version changes during export.
- `FM-DTB-012`: import audit event lacks consent proof hash.
- `FM-DTB-013`: idempotent retry creates multiple work copies.
- `FM-DTB-014`: consent revocation deletes already-imported work copy.
- `FM-DTB-015`: consent revocation fails to block future exports.
- `FM-DTB-016`: personal audit view hides destination tenant.
- `FM-DTB-017`: work audit view leaks personal Drive listing.
- `FM-DTB-018`: export proceeds without governance decision id.
- `FM-DTB-019`: source hash not preserved in work copy provenance.
- `FM-DTB-020`: boundary summary is computed from local drive state only.

## Cross-µservice Handoff Validation

- `HANDOFF-IDENTITY-DRIVE-OPENAPI`: drive honors active tenant claim from identity token.
- `HANDOFF-IDENTITY-GOVERNANCE-OPENAPI`: governance consent ceremony receives both memberships from identity.
- `HANDOFF-GOVERNANCE-DRIVE-OPENAPI`: drive copy endpoint requires export consent id and Cedar decision id.
- `HANDOFF-DRIVE-GOVERNANCE-ASYNCAPI`: drive emits export-read event consumed by governance boundary summary.
- `HANDOFF-DRIVE-AUDIT-PROTO`: personal export event proto includes source hash and destination tenant.
- `HANDOFF-GOVERNANCE-AUDIT-PROTO`: consent event proto includes one-file scope.
- `HANDOFF-AUDIT-GOVERNANCE-OPENAPI`: governance summary validates personal and work audit proof hashes.
- `HANDOFF-CEDAR`: `forbid-work-admin-pull-personal-drive` is evaluated on read and list attempts.
- `HANDOFF-PROVENANCE`: work copy provenance contract is hash-and-proof only, not content grant.
- `HANDOFF-IDEMPOTENCY`: copy request idempotency binds to one work copy id.
- `HANDOFF-TRACE`: trace id is preserved across identity, governance, drive, and audit-chain.
- `HANDOFF-ERROR`: personal-boundary denies use `403` without existence leak.
- `HANDOFF-REVOKE`: consent revocation event is consumed by drive before second export attempt.
- `HANDOFF-PRIVACY`: work audit projection redacts personal file path.
- `HANDOFF-REPLAY`: audit replay reconstructs consent, copy, read, and denial sequence.

## SLO Conformance

- `SLO-PASSKEY-SESSION-P95`: passkey session P95 <= 450 ms excluding authenticator wait.
- `SLO-TENANT-SWITCH-P95`: tenant context switch P95 <= 200 ms.
- `SLO-PERSONAL-METADATA-P95`: personal file metadata read P95 <= 250 ms.
- `SLO-CONSENT-CREATE-P95`: export consent creation P95 <= 350 ms.
- `SLO-CEDAR-EVALUATE-P95`: receive-side governance evaluation P95 <= 250 ms.
- `SLO-DRIVE-COPY-P95`: cross-tenant copy P95 <= 1200 ms for <= 25 MB fixture.
- `SLO-AUDIT-APPEND-P99`: each audit append P99 <= 150 ms.
- `SLO-BOUNDARY-DENY-P95`: personal-boundary denial P95 <= 200 ms.
- `SLO-SUMMARY-P95`: boundary summary P95 <= 800 ms.
- `SLO-AVAILABILITY`: drive, identity, governance, and audit-chain endpoints target 99.95 percent monthly availability.
- `SLO-THROUGHPUT`: one tenant supports 100 concurrent one-file exports without consent-scope collision.
- `SLO-PRIVACY`: zero personal Drive listing entries in work-tenant output.

## Reproducibility

- Named cargo test invocation: `cargo test -p oya-cross-msvc-tests dual_tenant_document_export -- --ignored --exact --test-threads=1`.
- Named deterministic seed: `CMIT-SEED-2026-05-20-DUAL-TENANT-EXPORT-CHRIS`.
- Required fixture bundle: `fixtures/cross-msvc/dual-tenant-export-chris-northstar.yaml`.
- Required policy bundle: `cedar-bundle-2026-05-20-cross-msvc`.
- Required source object hash: `sha256:personal-portfolio-2026`.
- Required clock: `2026-05-20T14:00:00Z`.
- Required teardown: delete work copy only after audit proof export.
- Test isolation: personal file fixture remains unchanged across the run.
- Stop condition: work copy exists, personal source remains unchanged, and all personal-boundary denials are observed.

## References

- `docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md`.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md`.
- `docs/decisions/ADR-0243-cedar-as-universal-gate.md`.
- `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`.
- `docs/decisions/ADR-0263-observability-emission-contract.md`.
- `docs/decisions/ADR-0276-backup-portability-gdpr-art-20.md`.
- `docs/decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md`.
- `docs/decisions/ADR-0312-court-warrant-scoped-piercing.md`.
- `docs/standards/documentation-rigor.md` section 3.2.5 rows 18, 21, 23, 25.
- `microservices/drive/contracts/openapi-v1.yaml`.
- `microservices/identity/contracts/openapi-v1.yaml`.
- `microservices/governance/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/asyncapi-v1.yaml`.
- `microservices/audit-chain/contracts/audit-event-v1.proto`.
