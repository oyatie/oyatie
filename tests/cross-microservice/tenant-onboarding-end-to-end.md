---
doc_class: CrossMicroserviceIntegrationTest
scenario_id: CMIT-001-tenant-onboarding-end-to-end
microservices_under_test:
  - tenancy
  - identity
  - workplace-integration
  - drive
  - mail
status: draft-canonical
date: 2026-05-20
owner: codex-cross-msvc-integration-tests-w1
related_oyatie_adrs:
  - ADR-0113-vcs-orchestrator-end-to-end
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
---

# Tenant Onboarding End To End

## Scenario Description

Mina Park, the named IT administrator for `tenant-haneul-biotech-us`, provisions a new regulated work tenant from a signed sales order and expects a complete chain from `tenancy` tenant shell, through `identity` membership and passkey enrollment, into `workplace-integration` directory binding, followed by default `drive` root-space creation and `mail` domain activation without any hidden manual step.

## Pre-conditions

### Named tenant fixtures

- Fixture tenant slug: `tenant-haneul-biotech-us`.
- Fixture legal name: `Haneul Biotech USA, Inc.`
- Fixture tenant class: `B2B_WORK_TENANT`.
- Fixture primary region: `us-east-1-cell-a`.
- Fixture DR pair: `us-west-2-cell-b`.
- Fixture onboarding principal: `principal-mina-park-admin`.
- Fixture onboarding email: `mina.park@haneulbio.example`.
- Fixture root domain: `haneulbio.example`.
- Fixture directory provider: `workplace-provider-google-workspace`.
- Fixture directory external id: `gw-haneul-880177`.
- Fixture tenant status before test: absent.
- Fixture legal basis: `commercial-contract-and-admin-consent`.
- Fixture data residency mode: `US_ONLY`.
- Fixture idempotency key: `idem-cmit-001-tenant-shell`.
- Fixture trace id prefix: `trace-cmit-001`.

### Named Cedar permits

- `permit-tenant-create-b2b-work-admin`.
- `permit-identity-bootstrap-admin`.
- `permit-workplace-directory-bind`.
- `permit-drive-root-provision`.
- `permit-mail-domain-activate`.
- `forbid-personal-tenant-read-from-work-admin`.
- `permit-audit-seal-onboarding`.
- `permit-pack-us-hipaa-baseline-evaluate`.

### Named pack activations

- `pack-SOC2-Type-II-baseline`.
- `pack-HIPAA-workforce-lite`.
- `pack-US-state-privacy-baseline`.
- `pack-domain-email-security-baseline`.
- `pack-ADR-0311-work-personal-boundary`.

### Starting state checks

- The tenancy lookup for `tenant-haneul-biotech-us` returns `404`.
- The identity lookup for `principal-mina-park-admin` returns `404`.
- The drive root lookup for `tenant-haneul-biotech-us` returns `404`.
- The mail domain lookup for `haneulbio.example` returns `404`.
- The workplace connector lookup for `gw-haneul-880177` returns `404`.
- The audit-chain stream `tenant-haneul-biotech-us.onboarding` exists as an empty sealed stream.
- The active policy bundle version is `cedar-bundle-2026-05-20-cross-msvc`.
- The integration harness clock seed is frozen at `2026-05-20T14:00:00Z`.

## Test Steps

1. Create the tenant shell in `tenancy`.
   - API call: `POST /tenancy/v1/tenants` with `TenantCreateFixture`.
   - Expected response: `201 Created` with `tenant_id="tenant-haneul-biotech-us"` and `lifecycle_state="PENDING_IDENTITY_BOOTSTRAP"`.
   - Assertion: response carries `X-Oya-Tenant-Context: tenant-haneul-biotech-us`.

2. Re-submit the tenant shell request with the same idempotency key.
   - API call: `POST /tenancy/v1/tenants` with `Idempotency-Key: idem-cmit-001-tenant-shell`.
   - Expected response: `200 OK` with the same `tenant_version=1`.
   - Assertion: no duplicate tenant row and no second `TenantCreated` audit event.

3. Fetch tenant policy obligations.
   - API call: `GET /tenancy/v1/tenants/tenant-haneul-biotech-us/policy-obligations`.
   - Expected response: `200 OK` with required permits `permit-identity-bootstrap-admin`, `permit-drive-root-provision`, and `permit-mail-domain-activate`.
   - Assertion: every obligation names the originating ADR and the active compliance pack.

4. Bootstrap the first identity principal.
   - API call: `POST /identity/v1/tenants/tenant-haneul-biotech-us/principals/bootstrap-admin`.
   - Expected response: `201 Created` with `principal_id="principal-mina-park-admin"` and `audience_type="B2B_TENANT_ADMIN"`.
   - Assertion: session token includes `tenant_id=tenant-haneul-biotech-us` and not a personal tenant id.

5. Enroll Mina's passkey ceremony.
   - API call: `POST /identity/v1/principals/principal-mina-park-admin/passkeys`.
   - Expected response: `201 Created` with `passkey_credential_id="passkey-mina-2026-05"` and `phishing_resistant=true`.
   - Assertion: returned token is bound to `work_tenant_context` and cannot enumerate personal tenant memberships.

6. Confirm ADR-0311 personal-boundary default deny.
   - API call: `POST /identity/v1/sessions/tenant-switch` targeting `personal-mina-park`.
   - Expected response: `403 Forbidden` with `error.code="TENANT_MEMBERSHIP_NOT_GRANTED"`.
   - Assertion: audit-chain emits `TenantBoundary.WorkPersonalRead` with outcome `DENY`.

7. Bind the Google Workspace directory.
   - API call: `POST /workplace-integration/v1/tenants/tenant-haneul-biotech-us/directories`.
   - Expected response: `202 Accepted` with `binding_id="dirbind-haneul-google-001"` and `state="VERIFYING_DOMAIN"`.
   - Assertion: workplace-integration validates `permit-workplace-directory-bind` through governance before persisting.

8. Publish directory-verification challenge.
   - API call: `POST /workplace-integration/v1/directory-bindings/dirbind-haneul-google-001/challenges`.
   - Expected response: `201 Created` with `dns_txt_name="_oya-directory.haneulbio.example"`.
   - Assertion: challenge expiry is at most 900 seconds and audit event records `DirectoryChallengeIssued`.

9. Mark directory verification complete.
   - API call: `POST /workplace-integration/v1/directory-bindings/dirbind-haneul-google-001/verify`.
   - Expected response: `200 OK` with `state="ACTIVE"` and `external_directory_id="gw-haneul-880177"`.
   - Assertion: state transition emits AsyncAPI event `workplace.directory.activated.v1`.

10. Create mapped identity groups from the directory.
    - API call: `POST /identity/v1/tenants/tenant-haneul-biotech-us/groups/import` referencing `dirbind-haneul-google-001`.
    - Expected response: `202 Accepted` with imported groups `haneul-admins`, `haneul-workforce`, and `haneul-compliance`.
    - Assertion: group membership source is immutable and references workplace binding version `1`.

11. Provision default drive root.
    - API call: `POST /drive/v1/tenants/tenant-haneul-biotech-us/root-spaces`.
    - Expected response: `201 Created` with `space_id="drive-root-haneul-001"` and `classification="WORK_TENANT"`.
    - Assertion: drive root policy includes `forbid-personal-tenant-read-from-work-admin`.

12. Create drive compliance folders.
    - API call: `POST /drive/v1/spaces/drive-root-haneul-001/folders:batchCreate`.
    - Expected response: `207 Multi-Status` with `People`, `Clinical-Ops`, `Finance`, and `Legal-Hold` all created.
    - Assertion: every folder inherits `tenant_id` and `pack-HIPAA-workforce-lite`.

13. Request mail domain activation.
    - API call: `POST /mail/v1/tenants/tenant-haneul-biotech-us/domains`.
    - Expected response: `202 Accepted` with `domain_id="mail-domain-haneulbio-example"` and `state="PENDING_DNS"`.
    - Assertion: response includes SPF, DKIM, DMARC, and BIMI setup requirements.

14. Publish DKIM selector material.
    - API call: `POST /mail/v1/domains/mail-domain-haneulbio-example/dkim-selectors`.
    - Expected response: `201 Created` with selector `oya202605` and `key_ref="openbao://mail/haneulbio/dkim/oya202605"`.
    - Assertion: private key material never appears in response or audit payload.

15. Complete mail DNS verification.
    - API call: `POST /mail/v1/domains/mail-domain-haneulbio-example/verify-dns`.
    - Expected response: `200 OK` with `state="ACTIVE"` and `dmarc_policy="quarantine"`.
    - Assertion: `MailDomainActivated` event carries tenant id and directory binding id.

16. Issue first tenant admin mailbox.
    - API call: `POST /mail/v1/tenants/tenant-haneul-biotech-us/mailboxes`.
    - Expected response: `201 Created` with `mailbox_id="mailbox-mina-park"` and address `mina.park@haneulbio.example`.
    - Assertion: mailbox ownership class is `WORK_TENANT`, not `PERSONAL_TENANT`.

17. Create welcome drive document.
    - API call: `POST /drive/v1/spaces/drive-root-haneul-001/documents`.
    - Expected response: `201 Created` with `document_id="doc-haneul-welcome-runbook"`.
    - Assertion: document creation emits `DriveObjectCreated` and references the admin principal.

18. Send onboarding completion mail.
    - API call: `POST /mail/v1/messages:send` from `mailbox-mina-park`.
    - Expected response: `202 Accepted` with `message_id="msg-haneul-onboarding-complete"`.
    - Assertion: mail event references the drive document id and the same trace id.

19. Query onboarding aggregate status.
    - API call: `GET /tenancy/v1/tenants/tenant-haneul-biotech-us/onboarding-status`.
    - Expected response: `200 OK` with all five domains marked `COMPLETE`.
    - Assertion: status cannot be `COMPLETE` unless tenancy, identity, workplace, drive, and mail checkpoints are present.

20. Read the unified audit trail.
    - API call: `GET /audit-chain/v1/streams/tenant-haneul-biotech-us.onboarding/events?trace_id=trace-cmit-001`.
    - Expected response: `200 OK` with at least 18 ordered events.
    - Assertion: every event has a Merkle inclusion proof and the same tenant id.

21. Verify no personal tenant artifact was created.
    - API call: `GET /tenancy/v1/principals/principal-mina-park-admin/personal-artifacts?trace_id=trace-cmit-001`.
    - Expected response: `200 OK` with `items=[]`.
    - Assertion: onboarding did not silently provision personal mail, personal drive, or consumer marketplace state.

22. Replay the AsyncAPI onboarding event stream.
    - API call: `GET /eventing/v1/replay?topic=tenant.onboarding.v1&trace_id=trace-cmit-001`.
    - Expected response: `200 OK` with deterministic event order and no gaps.
    - Assertion: event versions match `tenant.onboarding.v1`, `identity.principal.created.v1`, `workplace.directory.activated.v1`, `drive.root.created.v1`, and `mail.domain.activated.v1`.

23. Confirm SLO labels are attached.
    - API call: `GET /observability/v1/traces/trace-cmit-001/slo-labels`.
    - Expected response: `200 OK` with `scenario_id="CMIT-001-tenant-onboarding-end-to-end"`.
    - Assertion: every service span includes `tenant_id`, `cell_id`, `region`, and `policy_bundle_version`.

24. Finalize onboarding.
    - API call: `POST /tenancy/v1/tenants/tenant-haneul-biotech-us/onboarding:finalize`.
    - Expected response: `200 OK` with `lifecycle_state="ACTIVE"`.
    - Assertion: finalization fails if any downstream service checkpoint is missing or policy-forbidden.

## Test Data Fixtures

### Fixture `TenantCreateFixture`

```json
{
  "tenant_id": "tenant-haneul-biotech-us",
  "legal_name": "Haneul Biotech USA, Inc.",
  "tenant_class": "B2B_WORK_TENANT",
  "primary_region": "us-east-1-cell-a",
  "dr_pair_region": "us-west-2-cell-b",
  "residency_mode": "US_ONLY",
  "requested_packs": [
    "pack-SOC2-Type-II-baseline",
    "pack-HIPAA-workforce-lite",
    "pack-US-state-privacy-baseline"
  ],
  "sales_order_ref": "so-haneul-2026-05-20-001"
}
```

### Fixture `BootstrapAdminFixture`

```json
{
  "principal_id": "principal-mina-park-admin",
  "display_name": "Mina Park",
  "email": "mina.park@haneulbio.example",
  "audience_type": "B2B_TENANT_ADMIN",
  "tenant_memberships": [
    {
      "tenant_id": "tenant-haneul-biotech-us",
      "role": "TENANT_ADMIN",
      "source": "signed_sales_order"
    }
  ],
  "personal_tenant_link": null
}
```

### Fixture `WorkplaceDirectoryFixture`

```json
{
  "binding_id": "dirbind-haneul-google-001",
  "provider": "GOOGLE_WORKSPACE",
  "external_directory_id": "gw-haneul-880177",
  "domain": "haneulbio.example",
  "sync_groups": [
    "haneul-admins",
    "haneul-workforce",
    "haneul-compliance"
  ],
  "sync_mode": "SCIM_PULL_THEN_WEBHOOK"
}
```

### Fixture `DriveRootFixture`

```json
{
  "space_id": "drive-root-haneul-001",
  "tenant_id": "tenant-haneul-biotech-us",
  "tenant_ownership_class": "WORK_TENANT",
  "default_retention_policy": "hipaa-workforce-7y",
  "folders": [
    "People",
    "Clinical-Ops",
    "Finance",
    "Legal-Hold"
  ],
  "cedar_policy_ref": "policy/tenant-boundary-work-vs-personal.cedar"
}
```

### Fixture `MailDomainFixture`

```yaml
domain_id: mail-domain-haneulbio-example
tenant_id: tenant-haneul-biotech-us
domain: haneulbio.example
spf: "v=spf1 include:_spf.oyatie.example -all"
dkim_selector: oya202605
dmarc_policy: quarantine
bimi_required: true
mailbox_seed:
  - mina.park@haneulbio.example
```

### Fixture `ExpectedAuditEvents`

```yaml
events:
  - TenantCreated
  - TenantPolicyObligationsEvaluated
  - IdentityBootstrapAdminCreated
  - PasskeyCredentialEnrolled
  - TenantBoundaryWorkPersonalRead
  - WorkplaceDirectoryChallengeIssued
  - WorkplaceDirectoryActivated
  - IdentityGroupsImported
  - DriveRootCreated
  - DriveFolderCreated
  - MailDomainRequested
  - MailDkimSelectorCreated
  - MailDomainActivated
  - MailboxCreated
  - DriveObjectCreated
  - MailMessageAccepted
  - TenantOnboardingAggregateComplete
  - TenantActivated
```

## Assertion Catalogue

### What passes

- `PASS-TENANT-001`: tenant shell is created exactly once.
- `PASS-TENANT-002`: idempotent retry returns the original tenant version.
- `PASS-TENANT-003`: tenant lifecycle cannot skip identity bootstrap.
- `PASS-IDENTITY-001`: bootstrap admin principal carries `B2B_TENANT_ADMIN`.
- `PASS-IDENTITY-002`: passkey enrollment binds to the work tenant context.
- `PASS-IDENTITY-003`: tenant switch to non-member personal tenant is denied.
- `PASS-WORKPLACE-001`: directory binding has active verification proof.
- `PASS-WORKPLACE-002`: imported groups cite the binding version.
- `PASS-DRIVE-001`: drive root has ownership class `WORK_TENANT`.
- `PASS-DRIVE-002`: default folders inherit HIPAA retention.
- `PASS-MAIL-001`: domain activation requires DNS proof.
- `PASS-MAIL-002`: DKIM private key remains secret.
- `PASS-MAIL-003`: first mailbox is work-owned.
- `PASS-AUDIT-001`: every service emits a sealed audit event.
- `PASS-AUDIT-002`: every event carries the same trace id.
- `PASS-POLICY-001`: every mutation passes a Cedar permit evaluation.
- `PASS-POLICY-002`: personal-boundary default-deny is explicitly observed.
- `PASS-SLO-001`: aggregate onboarding completes within the scenario budget.
- `PASS-REPLAY-001`: AsyncAPI replay is gap-free.
- `PASS-FINALIZE-001`: final tenant state is `ACTIVE`.

### What fails

- `FAIL-TENANT-001`: duplicate tenant row on idempotent retry.
- `FAIL-TENANT-002`: finalization succeeds with missing downstream checkpoint.
- `FAIL-IDENTITY-001`: admin principal lacks tenant-scoped audience type.
- `FAIL-IDENTITY-002`: session token can enumerate personal tenant state.
- `FAIL-WORKPLACE-001`: directory binding activates without DNS proof.
- `FAIL-WORKPLACE-002`: group import lacks immutable source binding.
- `FAIL-DRIVE-001`: root space omits `tenant_ownership_class`.
- `FAIL-DRIVE-002`: folders omit inherited compliance packs.
- `FAIL-MAIL-001`: mail domain activates without SPF, DKIM, and DMARC.
- `FAIL-MAIL-002`: DKIM private key appears in response.
- `FAIL-AUDIT-001`: any emitted event lacks Merkle proof.
- `FAIL-AUDIT-002`: trace id changes across microservice boundary.
- `FAIL-POLICY-001`: mutation occurs without Cedar decision id.
- `FAIL-POLICY-002`: work admin gains personal-tenant read.
- `FAIL-SLO-001`: P95 onboarding latency exceeds the declared budget.

## Failure Mode Coverage

- `FM-ONBOARD-001`: duplicate tenant creation under retry pressure.
- `FM-ONBOARD-002`: identity principal created without tenant membership.
- `FM-ONBOARD-003`: passkey enrollment binds to the wrong tenant.
- `FM-ONBOARD-004`: directory verification bypasses DNS challenge.
- `FM-ONBOARD-005`: workplace group import trusts unverified external ids.
- `FM-ONBOARD-006`: drive root is created as personal-owned by mistake.
- `FM-ONBOARD-007`: drive default folder retention does not inherit pack activation.
- `FM-ONBOARD-008`: mail domain is usable before DKIM proof.
- `FM-ONBOARD-009`: DKIM private material leaks through response serialization.
- `FM-ONBOARD-010`: onboarding aggregate reports complete with one service missing.
- `FM-ONBOARD-011`: audit-chain order differs from eventing replay order.
- `FM-ONBOARD-012`: trace id not propagated from tenancy to mail.
- `FM-ONBOARD-013`: Cedar permit version drift between services.
- `FM-ONBOARD-014`: ADR-0311 personal boundary silently not evaluated.
- `FM-ONBOARD-015`: compliance pack activation omitted from drive or mail.
- `FM-ONBOARD-016`: DR pair is not recorded in the tenant shell.
- `FM-ONBOARD-017`: lifecycle finalization races ahead of async directory activation.
- `FM-ONBOARD-018`: mailbox creation succeeds for a non-tenant member.
- `FM-ONBOARD-019`: audit stream exists but has unsealed onboarding events.
- `FM-ONBOARD-020`: status endpoint uses local service truth instead of aggregate checkpoints.

## Cross-µservice Handoff Validation

- `HANDOFF-TENANCY-IDENTITY-OPENAPI`: `POST /tenancy/v1/tenants` response field `tenant_id` conforms to identity `TenantMembership.tenant_id`.
- `HANDOFF-TENANCY-IDENTITY-ASYNCAPI`: `tenant.created.v1` event is consumed by identity with `tenant_version=1`.
- `HANDOFF-IDENTITY-WORKPLACE-OPENAPI`: identity group import request accepts only active workplace `binding_id`.
- `HANDOFF-WORKPLACE-IDENTITY-ASYNCAPI`: `workplace.directory.activated.v1` contains the external directory id used by identity import.
- `HANDOFF-TENANCY-DRIVE-OPENAPI`: drive root creation requires an active tenant state or pending onboarding state with drive obligation.
- `HANDOFF-DRIVE-AUDIT-PROTO`: `DriveRootCreated` proto carries `tenant_ownership_class`.
- `HANDOFF-TENANCY-MAIL-OPENAPI`: mail domain activation accepts the tenant root domain registered in tenancy.
- `HANDOFF-MAIL-AUDIT-PROTO`: `MailDomainActivated` proto includes DKIM selector without private material.
- `HANDOFF-ALL-AUDIT-ASYNCAPI`: every service publishes to `audit.events.v1` with shared `trace_id`.
- `HANDOFF-ALL-OBSERVABILITY`: trace propagation uses W3C `traceparent` and `X-Oya-Tenant-Context`.
- `HANDOFF-POLICY-CEDAR`: every service stores `cedar_decision_id` for each mutation.
- `HANDOFF-IDEMPOTENCY`: tenancy idempotency key is not reused by downstream services.
- `HANDOFF-CLOCK`: all service timestamps are HLC-compatible and monotonic under frozen seed.
- `HANDOFF-ERROR`: downstream denial propagates as aggregate `ONBOARDING_BLOCKED`, not partial success.
- `HANDOFF-REPLAY`: eventing replay reconstructs the same aggregate status as the tenancy status endpoint.

## SLO Conformance

- `SLO-TENANT-CREATE-P95`: tenant shell creation P95 <= 250 ms.
- `SLO-IDENTITY-BOOTSTRAP-P95`: bootstrap admin creation P95 <= 300 ms.
- `SLO-PASSKEY-CEREMONY-P95`: passkey ceremony API P95 <= 450 ms excluding human authenticator wait.
- `SLO-DIRECTORY-BIND-P95`: directory binding request P95 <= 400 ms.
- `SLO-DIRECTORY-VERIFY-P95`: DNS verification callback P95 <= 900 ms after challenge visibility.
- `SLO-DRIVE-ROOT-P95`: drive root creation P95 <= 500 ms.
- `SLO-MAIL-DOMAIN-P95`: mail domain verification P95 <= 1200 ms.
- `SLO-AUDIT-EMIT-P99`: audit event append P99 <= 150 ms per service.
- `SLO-ONBOARDING-AGGREGATE-P95`: aggregate happy path P95 <= 10 seconds after DNS proofs are present.
- `SLO-AVAILABILITY`: each service endpoint in this scenario targets 99.95 percent monthly availability.
- `SLO-THROUGHPUT`: harness supports 50 concurrent tenant onboardings per cell without shared-state collision.
- `SLO-EVENT-REPLAY`: replay of one trace returns within 2 seconds for <= 50 events.

## Reproducibility

- Named cargo test invocation: `cargo test -p oya-cross-msvc-tests tenant_onboarding_end_to_end -- --ignored --exact --test-threads=1`.
- Named deterministic seed: `CMIT-SEED-2026-05-20-TENANT-ONBOARDING-HANEUL`.
- Required fixture bundle: `fixtures/cross-msvc/tenant-onboarding-haneul-biotech.yaml`.
- Required policy bundle: `cedar-bundle-2026-05-20-cross-msvc`.
- Required clock: `2026-05-20T14:00:00Z`.
- Required cell topology: `us-east-1-cell-a` with DR pair `us-west-2-cell-b`.
- Required replay mode: audit-chain replay and eventing replay both enabled.
- Test isolation: tenant slug is deleted only by test teardown after audit export is captured.
- Stop condition: all `PASS-*` assertions pass and no `FAIL-*` criteria are observed.

## References

- `docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md`.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md`.
- `docs/decisions/ADR-0243-cedar-as-universal-gate.md`.
- `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`.
- `docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md`.
- `docs/decisions/ADR-0263-observability-emission-contract.md`.
- `docs/decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md`.
- `docs/standards/documentation-rigor.md` section 3.2.5.
- `microservices/tenancy/contracts/openapi-v1.yaml`.
- `microservices/identity/contracts/openapi-v1.yaml`.
- `microservices/workplace-integration/contracts/openapi-v1.yaml`.
- `microservices/drive/contracts/openapi-v1.yaml`.
- `microservices/mail/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/asyncapi-v1.yaml`.
- `microservices/audit-chain/contracts/audit-event-v1.proto`.
