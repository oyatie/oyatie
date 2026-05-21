---
doc_class: CrossMicroserviceHandoffMatrix
title: "Developer SDK Cross-Microservice Handoff Matrix"
status: Draft
date: 2026-05-20
microservice: developer-sdk
owner_team: axis-ecosystem
---

# Developer SDK Cross-Microservice Handoff Matrix

This matrix records concrete handoffs for the `developer-sdk` microservice.
REST shapes are from `microservices/developer-sdk/contracts/openapi/developer-sdk.yaml`.
The ecosystem aggregate surface is `microservices/developer-sdk/contracts/openapi/oya-ecosystem.yaml`.
Async shapes are from `microservices/developer-sdk/contracts/asyncapi/developer-sdk-events.yaml`.
gRPC names are from `microservices/developer-sdk/contracts/proto/developer_sdk.proto`.
Cedar policies are from `microservices/developer-sdk/policies/`.
Audit-chain emission follows ADR-0263 with `source_microservice=developer-sdk`.

## Inbound Callers

| # | Calling microservice | Named API on `developer-sdk` | Data shape | Cedar permit required | Audit event emitted |
|---|---|---|---|---|---|
| 1 | `application` | `GET /submissions/{submissionId}/status` `streamSubmissionStatus` | `VettingStageEvent` | `developer-scope.cedar` action `DeveloperSdk::read_submission_status` | `DeveloperSubmissionStatusRead` |
| 2 | `application` | `GET /sdk/families/{family}/{version}/download` `downloadSdk` | SDK artifact response | `public-read.cedar` action `download_sdk` on `PublicSdkArtifact` | `DeveloperSdkArtifactDownloaded` |
| 3 | `api-gateway` | `GET /sdk/families` `listSdkFamilies` | `SdkFamily[]` | `public-read.cedar` action `list_sdk_families` | `DeveloperSdkFamiliesListed` |
| 4 | `api-gateway` | `GET /sdk/families/{family}/{version}/download` `downloadSdk` | SDK artifact response | `public-read.cedar` action `download_sdk` | `DeveloperSdkArtifactDownloaded` |
| 5 | `cloud-secrets` | `POST /developers/{developerId}/signing-keys/{keyId}/revoke` `revokeSigningKey` | signing key revocation request | `developer-scope.cedar` action `DeveloperSdk::revoke_signing_key` | `SigningKeyRevoked` |
| 6 | `cloud-iac` | `POST /submissions/{submissionId}/status` internal stage update | `VettingStageEvent` | `admin-scope.cedar` action `DeveloperSdk::update_submission_stage` | `DeveloperSubmissionStageUpdated` |
| 7 | `payments` | `GET /payout/balance` `getPayoutBalance` | `PayoutBalance` | `payout-scope.cedar` action `DeveloperSdk::read_payout_balance` | `DeveloperPayoutBalanceRead` |
| 8 | `payments` | `GET /payout/ledger` `streamPayoutLedger` | `PayoutLedgerEntry[]` | `payout-scope.cedar` action `DeveloperSdk::read_payout_ledger` | `DeveloperPayoutLedgerRead` |
| 9 | `audit-chain` | `GET /developers/{developerId}/signing-keys` `listSigningKeys` | `SigningKey[]` | `auditor-scope.cedar` action `DeveloperSdk::read_signing_keys_for_audit` | `DeveloperSigningKeysReadForAudit` |
| 10 | `ops-dashboard-control-center` | `GET /developers/{developerId}/kyc/status` inline operator read | `Developer` KYC projection | `admin-scope.cedar` action `DeveloperSdk::read_kyc_operator` | `DeveloperKycStatusRead` |
| 11 | `cloud-iac` | `POST /sandbox` `provisionSandbox` | `SandboxTenant` | `admin-scope.cedar` action `DeveloperSdk::provision_sandbox_internal` | `SandboxProvisioned` |
| 12 | `application` | `POST /submissions` `submitPluginVersion` | `SubmitRequest` | `developer-scope.cedar` action `DeveloperSdk::submit_plugin_version` | `DeveloperSubmissionCreated` |
| 13 | `api-gateway` | `POST /developers` `signupDeveloper` | `DeveloperSignupRequest` | `public-read.cedar` plus abuse preflight `DeveloperSdk::signup_developer` | `DeveloperSignupStarted` |
| 14 | `application` | `GET /tax-forms/{year}` `downloadTaxForm` | tax-form artifact response | `payout-scope.cedar` action `DeveloperSdk::download_tax_form` | `DeveloperTaxFormDownloaded` |

## Outbound Callees

| # | Callee microservice | Named API called by `developer-sdk` | Data shape sent or received | Cedar permit required | Audit event consumed |
|---|---|---|---|---|---|
| 1 | `audit-chain` | `POST /emit` `emitEvent` | `AuditEvent` with `source_microservice=developer-sdk` | `developer-scope.cedar` action `AuditChain::emit_developer_event` | consumes `DeveloperSdkAuditReceiptAccepted` |
| 2 | `cloud-secrets` | `GET /secrets/{tenant}/developer-sdk/signing-key/reference` `getSecretReference` | `SecretReference` | `developer-scope.cedar` action `CloudSecrets::read_developer_signing_reference` | consumes `CloudSecretsDeveloperSigningReferenceRead` |
| 3 | `cloud-secrets` | `GET /secrets/{tenant}/developer-sdk/payout-token/reference` | `SecretReference` | `payout-scope.cedar` action `CloudSecrets::read_developer_payout_reference` | consumes `CloudSecretsReferenceRead` |
| 4 | `cell` | `GET /tenants/{tenant_id}/assignment` `getCellAssignment` | sandbox `CellAssignment` | `tenant-scope.cedar` action `Cell::resolve_sandbox_cell` | consumes `CellAssignmentResolvedForSandbox` |
| 5 | `api-gateway` | `POST /edge/admission` `admitEdgeRequest` | `EdgeAdmissionRequest` for sandbox endpoint | `route-authorization.cedar` action `Gateway::admit_sandbox_request` | consumes `ApiGatewaySandboxRequestAdmitted` |
| 6 | `cloud-iac` | `POST /charts/validate` `validateChartSignature` | `ChartSignatureValidationRequest` | `public-read.cedar` action `CloudIac::validate_chart_signature` | consumes `CloudIacChartSignatureValidated` |
| 7 | `cloud-iac` | `POST /microservices/developer-sdk/render` `triggerRender` | `RenderedManifest` for SDK docs portal | `ci-scope.cedar` action `CloudIac::render_developer_sdk_chart` | consumes `CloudIacRenderRequested` |
| 8 | `application` | `GET /modules/{module}/manifest` `get_module_manifest` | `ModuleManifest` | `developer-scope.cedar` action `Application::read_sandbox_manifest` | consumes `ApplicationSandboxManifestRead` |
| 9 | `payments` | `POST /v1/payouts` `schedulePayout` | `SchedulePayoutRequest` | `payout-scope.cedar` action `Payments::schedule_developer_payout` | consumes `PayoutScheduled` |
| 10 | `payments` | `POST /v1/sub-merchants` `onboardSubMerchant` | `OnboardSubMerchantRequest` | `payout-scope.cedar` action `Payments::onboard_developer_submerchant` | consumes `SubMerchantOnboarded` |
| 11 | `observability` | `POST /metrics/developer-sdk` | inline `DeveloperSdkMetric {developer_hash, stage, status}` | `public-read.cedar` action `Observability::write_developer_metric` | consumes `MetricAccepted` |
| 12 | `compliance` | `POST /kyc/evaluate` | inline `KycEvaluationRequest {developer_id, country, evidence_digest}` | `developer-scope.cedar` action `Compliance::evaluate_developer_kyc` | consumes `DeveloperKycEvaluated` |

## Event Subscriptions

| # | AsyncAPI channel subscribed | Event class | Handler behavior | Dead-letter policy |
|---|---|---|---|---|
| 1 | `cloud-secrets.secret.revoked` | `SecretRevokedPayload` | revokes affected developer signing or payout key metadata | retry 8 times, then `developer-sdk.dlq.secret_revoked` |
| 2 | `cloud-secrets.secret.rotated` | `SecretLifecyclePayload` | reloads signing-key and payout-token references | retry 8 times, then `developer-sdk.dlq.secret_rotated` |
| 3 | `workflow-events/cell.assigned` | `CellAssignedPayload` | binds sandbox tenant to assigned cell | retry 8 times, then `developer-sdk.dlq.cell_assigned` |
| 4 | `cloud-iac.apply.completed` | `ApplyCompletedPayload` | updates SDK docs portal deployment state | retry 6 times, then `developer-sdk.dlq.iac_apply_completed` |
| 5 | `payments.payment-events.{tenant_id}` | `PayoutCompleted` | marks developer payout ledger entry settled | retry 8 times, then `developer-sdk.dlq.payout_completed` |
| 6 | `payments.payment-events.{tenant_id}` | `PayoutFailed` | marks developer payout ledger entry failed and emits payout deferred | retry 8 times, then `developer-sdk.dlq.payout_failed` |
| 7 | `payments.payment-events.{tenant_id}` | `SubMerchantRestricted` | pauses app-store submission monetization | retry 5 times, then `developer-sdk.dlq.submerchant_restricted` |
| 8 | `application.workflow-events/application.module.load.rejected` | `ModuleLoadRejected` | updates plugin submission vetting stage | retry 6 times, then `developer-sdk.dlq.module_load_rejected` |
| 9 | `audit-chain.audit.seal.minted` | `SealMintedPayload` | seals onboarding, signing-key, sandbox, and payout audit receipts | retry 10 times, then `developer-sdk.dlq.audit_seal_minted` |
| 10 | `api-gateway.upstream.circuit-open` | `GatewayCircuitOpen` | pauses sandbox endpoint activation | retry 4 times, then `developer-sdk.dlq.gateway_circuit_open` |

## Event Emissions

| # | AsyncAPI channel published | Event class | Payload schema | Downstream consumers |
|---|---|---|---|---|
| 1 | `oya.developer-sdk.onboarding` | `DeveloperOnboarded` | `developer-sdk-events.yaml#/components/schemas/DeveloperEvent` | `application`, `payments`, `audit-chain` |
| 2 | `oya.developer-sdk.onboarding` | `DeveloperKycPassed` | `DeveloperEvent` | `application`, `payments`, `cloud-secrets` |
| 3 | `oya.developer-sdk.onboarding` | `DeveloperKycRejected` | `DeveloperKycRejectedEvent` | `application`, `audit-chain`, `compliance` |
| 4 | `oya.developer-sdk.signing-key` | `SigningKeyIssued` | `SigningKeyEvent` | `cloud-secrets`, `audit-chain`, `application` |
| 5 | `oya.developer-sdk.signing-key` | `SigningKeyRevoked` | `SigningKeyEvent` | `cloud-secrets`, `application`, `audit-chain` |
| 6 | `oya.developer-sdk.sandbox` | `SandboxProvisioned` | `SandboxEvent` | `cell`, `api-gateway`, `application`, `audit-chain` |
| 7 | `oya.developer-sdk.sandbox` | `SandboxReset` | `SandboxEvent` | `cell`, `api-gateway`, `application` |
| 8 | `oya.developer-sdk.codegen` | `SdkCodegenEmitted` | `SdkCodegenEvent` | `application`, `cloud-iac`, `observability` |
| 9 | `oya.developer-sdk.payout` | `PayoutSettled` | `PayoutEvent` | `payments`, `audit-chain`, `application` |
| 10 | `oya.developer-sdk.payout` | `PayoutDeferred` | `PayoutEvent` | `payments`, `ops-dashboard-control-center`, `audit-chain` |
| 11 | `oya.developer-sdk.tax-form` | `TaxFormEmitted` | `TaxFormEvent` | `payments`, `audit-chain`, `application` |
| 12 | `audit-chain /emit` | `DeveloperSignupStarted` | `AuditEvent.payload` with `email_hash`, `country`, `developer_id` | `audit-chain` |
| 13 | `audit-chain /emit` | `DeveloperKycDecisionRecorded` | `AuditEvent.payload` with `developer_id`, `decision`, `evidence_digest` | `audit-chain`, `compliance` |
| 14 | `audit-chain /emit` | `DeveloperSigningKeyIssued` | `AuditEvent.payload` with `developer_id`, `key_id`, `public_key_fingerprint` | `audit-chain` |
| 15 | `audit-chain /emit` | `DeveloperSandboxProvisioned` | `AuditEvent.payload` with `developer_id`, `sandbox_tenant`, `cell_id` | `audit-chain`, `cell` |
| 16 | `audit-chain /emit` | `DeveloperPayoutScheduled` | `AuditEvent.payload` with `developer_id`, `payout_id`, `amount` | `audit-chain`, `payments` |

## Synchronous vs Asynchronous Boundaries

| # | Boundary | Mode | Reasoning |
|---|---|---|---|
| 1 | `signupDeveloper` | synchronous admission, asynchronous KYC | caller needs developer id; KYC is external and replayable |
| 2 | `submitIdDocument` | synchronous upload receipt | evidence digest must be returned before KYC evaluation |
| 3 | `submitLivenessProof` | synchronous upload receipt | liveness proof digest must bind to developer id |
| 4 | `addBankAccount` | synchronous tokenization | payout setup cannot continue without tokenized account reference |
| 5 | `issueSigningKey` | synchronous | developer needs key id and public fingerprint before signing |
| 6 | `revokeSigningKey` | synchronous admission, asynchronous consumer propagation | revocation must commit before cloud-secrets and application converge |
| 7 | `provisionSandbox` | synchronous admission, asynchronous cell/gateway/application activation | sandbox tenant id must exist before downstream setup |
| 8 | `resetSandbox` | synchronous admission, asynchronous reset work | reset id must be returned before cell cleanup |
| 9 | public SDK list/download | synchronous | callers need artifact response immediately |
| 10 | `submitPluginVersion` | synchronous admission, asynchronous vetting | submission id must exist before vetting stages |
| 11 | `streamSubmissionStatus` | synchronous | application and user interfaces need current stage |
| 12 | payout balance and ledger reads | synchronous | financial views need direct current data |
| 13 | payout scheduling to `payments` | synchronous | payout id and PSP acceptance are required |
| 14 | onboarding/signing/sandbox events | asynchronous | downstream services converge after developer state commits |
| 15 | audit emission for KYC and payouts | synchronous | ADR-0263 receipt required before state is durable |

## Failure Mode Cascade

| # | Failure in `developer-sdk` | Upstream impact | Circuit breaker | Retry policy |
|---|---|---|---|---|
| 1 | signup endpoint unavailable | api-gateway cannot onboard developers | `developer-signup` breaker returns 503 | retry safe POST only with idempotency key |
| 2 | KYC evidence upload fails | developer onboarding stalls | `developer-kyc` breaker marks review pending | retry upload by evidence digest |
| 3 | signing key issue fails | application cannot accept plugin signatures | `signing-key` breaker fails closed | retry with same key request id |
| 4 | signing key revoke fails | revoked keys may remain active | `signing-key-revoke` breaker raises urgent incident | retry revocation until cloud-secrets ack |
| 5 | sandbox provisioning fails | sandbox routes and cell assignment do not exist | `sandbox-provision` breaker blocks sandbox activation | retry with `sandbox_tenant` id |
| 6 | submission status write fails | application cannot show vetting progress | `submission-stage` breaker stores pending update | retry by `submissionId` and stage |
| 7 | payout ledger read fails | developer portal cannot show balance | `payout-ledger` breaker returns last sealed snapshot | retry background refresh |
| 8 | payments schedule payout fails | developer payout remains deferred | `payments-payout` breaker opens by developer | retry with same payout id |
| 9 | audit emit failure | onboarding, signing, sandbox, payout mutation refuses commit | `developer-audit` breaker fails closed | retry 10 times, then hold in `developer_sdk.audit_pending` |
| 10 | event bus failure | downstream services lag | outbox breaker spools events | replay by `event_id` |
| 11 | DLQ saturation | KYC, sandbox, and payout convergence delayed | `developer-dlq` breaker pauses noncritical submissions | manual replay after operator review |
| 12 | cloud-secrets signing reference unavailable | new keys cannot be issued | `developer-signing-secret` breaker fails closed | retry versioned reference read |

## Cross-tenant Coordination

| # | Scenario | Cedar guard pattern | Audit-mirror requirement |
|---|---|---|---|
| 1 | developer organization parent reads child developer payout | `payout-scope.cedar` with active conglomerate grant | mirror `ConglomerateParentReadAction` to parent and child partitions |
| 2 | sandbox tenant created in separate cell | `developer-scope.cedar` plus `Cell::resolve_sandbox_cell` | mirror `DeveloperSandboxProvisioned` to developer and sandbox tenants |
| 3 | cross-jurisdiction payout tax form | `payout-scope.cedar` with country and residency context | mirror `ConglomerateCrossJurisdictionResidencyEnforced` |
| 4 | office-scoped developer admin action | `OfficeBoundaryAttemptEvaluated` with `sub_scope_path` | mirror final office allow or deny |
| 5 | personal context attempts payout admin action | `payout-scope.cedar` forbids personal context | mirror `ConglomeratePersonalTenantBoundaryRefused` |

## Data Shape Ledger

| # | Shape | Source | Required handoff fields |
|---|---|---|---|
| 1 | `DeveloperSignupRequest` | `openapi/developer-sdk.yaml` | `email`, `legal_name`, `country` |
| 2 | `Developer` | `openapi/developer-sdk.yaml` | `developer_id`, `email_hash`, `country`, `kyc_status` |
| 3 | `BankAccountRequest` | `openapi/developer-sdk.yaml` | `developer_id`, `account_token`, `country` |
| 4 | `SigningKey` | `openapi/developer-sdk.yaml` | `key_id`, `developer_id`, `public_key_fingerprint`, `state` |
| 5 | `SandboxTenant` | `openapi/developer-sdk.yaml` | `sandbox_tenant`, `developer_id`, `cell_id`, `state` |
| 6 | `SdkFamily` | `openapi/developer-sdk.yaml` | `family`, `version`, `artifact_digest` |
| 7 | `PayoutBalance` | `openapi/developer-sdk.yaml` | `developer_id`, `available`, `pending`, `currency` |
| 8 | `PayoutLedgerEntry` | `openapi/developer-sdk.yaml` | `entry_id`, `payout_id`, `amount`, `state` |
| 9 | `SubmitRequest` | `openapi/developer-sdk.yaml` | `plugin_id`, `version`, `artifact_digest`, `signature` |
| 10 | `VettingStageEvent` | `openapi/developer-sdk.yaml` | `submissionId`, `stage`, `status`, `emitted_at` |

## Cedar Guard Ledger

| # | Policy file | Principal | Action | Resource |
|---|---|---|---|---|
| 1 | `developer-scope.cedar` | `Principal::developer` | `DeveloperSdk::submit_plugin_version` | `Submission::{submissionId}` |
| 2 | `developer-scope.cedar` | `Service::application` | `DeveloperSdk::read_submission_status` | `Submission::{submissionId}` |
| 3 | `developer-scope.cedar` | `Service::cloud-secrets` | `DeveloperSdk::revoke_signing_key` | `SigningKey::{keyId}` |
| 4 | `admin-scope.cedar` | `Service::cloud-iac` | `DeveloperSdk::update_submission_stage` | `Submission::{submissionId}` |
| 5 | `admin-scope.cedar` | `Service::cloud-iac` | `DeveloperSdk::provision_sandbox_internal` | `Sandbox::{sandbox_tenant}` |
| 6 | `payout-scope.cedar` | `Service::payments` | `DeveloperSdk::read_payout_balance` | `Developer::{developerId}` |
| 7 | `payout-scope.cedar` | `Service::developer-sdk` | `Payments::schedule_developer_payout` | `Payout::{payout_id}` |
| 8 | `public-read.cedar` | `Anonymous` | `download_sdk` | `PublicSdkArtifact::{family}/{version}` |
| 9 | `public-read.cedar` | `Anonymous` | `list_sdk_families` | `PublicSdkArtifact::*` |
| 10 | `auditor-scope.cedar` | `Service::audit-chain` | `DeveloperSdk::read_signing_keys_for_audit` | `Developer::{developerId}` |

## Audit Event Class Ledger

| # | Audit class | Emitting handoff | ADR-0263 envelope fields that must be present |
|---|---|---|---|
| 1 | `DeveloperSignupStarted` | `signupDeveloper` | `developer_id`, `email_hash`, `country`, `audit_id` |
| 2 | `DeveloperKycDecisionRecorded` | KYC evaluation result | `developer_id`, `decision`, `evidence_digest`, `audit_id` |
| 3 | `DeveloperSigningKeyIssued` | `issueSigningKey` | `developer_id`, `key_id`, `public_key_fingerprint`, `audit_id` |
| 4 | `SigningKeyRevoked` | `revokeSigningKey` | `developer_id`, `key_id`, `reason`, `audit_id` |
| 5 | `DeveloperSandboxProvisioned` | `provisionSandbox` | `developer_id`, `sandbox_tenant`, `cell_id`, `audit_id` |
| 6 | `DeveloperSubmissionCreated` | `submitPluginVersion` | `submissionId`, `plugin_id`, `artifact_digest`, `audit_id` |
| 7 | `DeveloperPayoutScheduled` | payout scheduling | `developer_id`, `payout_id`, `amount`, `audit_id` |
| 8 | `DeveloperTaxFormDownloaded` | `downloadTaxForm` | `developer_id`, `year`, `country`, `audit_id` |
| 9 | `ConglomerateParentReadAction` | parent payout read | `tenant_id`, `sub_scope_path`, `action`, `resource_ref` |

## Handoff Control Checklist

1. `signupDeveloper` must hash email before audit emission.
2. `signupDeveloper` must persist country for tax and KYC.
3. `submitIdDocument` must store evidence digest, not raw document in event payload.
4. `submitLivenessProof` must bind proof to developer id.
5. `addBankAccount` must store tokenized account only.
6. `issueSigningKey` must emit public fingerprint.
7. `issueSigningKey` must never emit private key material.
8. `revokeSigningKey` must notify cloud-secrets.
9. `listSigningKeys` for audit must omit private material.
10. `provisionSandbox` must call cell assignment before gateway activation.
11. `resetSandbox` must keep the same sandbox tenant id.
12. `listSdkFamilies` must be public-read only.
13. `downloadSdk` must be public-read and artifact-digest based.
14. `getPayoutBalance` must be payout-scoped.
15. `streamPayoutLedger` must be payout-scoped.
16. `downloadTaxForm` must be payout-scoped.
17. `submitPluginVersion` must require artifact digest.
18. `submitPluginVersion` must require plugin signature.
19. `streamSubmissionStatus` must expose only the caller's submission.
20. KYC passed must emit `DeveloperKycPassed`.
21. KYC rejected must emit `DeveloperKycRejected`.
22. Signing key issue must emit `SigningKeyIssued`.
23. Signing key revoke must emit `SigningKeyRevoked`.
24. Sandbox provision must emit `SandboxProvisioned`.
25. Sandbox reset must emit `SandboxReset`.
26. SDK code generation must emit `SdkCodegenEmitted`.
27. Payout settlement must emit `PayoutSettled`.
28. Payout failure must emit `PayoutDeferred`.
29. Tax form creation must emit `TaxFormEmitted`.
30. Cloud-secrets revocation must pause affected signing key.
31. Cloud-secrets rotation must reload payout token reference.
32. Cell assignment must bind sandbox to cell id.
33. Cloud-IAC apply completion must update docs portal state.
34. Payment payout completion must seal ledger entry.
35. Payment payout failure must mark payout deferred.
36. Sub-merchant restriction must pause monetization.
37. Module load rejection must update vetting stage.
38. Audit seal minted must close receipt status.
39. Gateway circuit open must pause sandbox route activation.
40. Developer events must be keyed by `developer_id`.
41. Submission events must be keyed by `submissionId`.
42. Payout events must be keyed by `payout_id`.
43. Signing-key events must be keyed by `key_id`.
44. Sandbox events must be keyed by `sandbox_tenant`.
45. Outbox replay must preserve developer order.
46. DLQ replay must not publish payout before KYC passed.
47. Payout schedule retries must preserve payout id.
48. Sandbox provision retries must preserve sandbox tenant id.
49. Submission stage retries must preserve stage name.
50. KYC retries must preserve evidence digest.
51. Public SDK metrics must not include developer ids.
52. Payout metrics must hash developer ids.
53. Audit events must include `source_microservice=developer-sdk`.
54. Audit events must include `trace_id`.
55. Audit events must include `span_id`.
56. Audit events must include `audit_id`.
57. Audit events must include `payload_data_class`.
58. Cross-tenant parent reads must mirror both tenant partitions.
59. Sandbox tenant events must mirror platform and sandbox partitions.
60. Cross-jurisdiction payout events must include country.
61. Office-scoped admin actions must include `sub_scope_path`.
62. Personal-context payout admin actions must be refused.
63. Developer signing key references must remain version-pinned.
64. Developer payout token references must remain version-pinned.
65. Chart signature validation must block plugin approval.
66. Application manifest read must block plugin publish.
67. Gateway admission must block sandbox route activation.
68. Compliance KYC denial must stop downstream payout setup.
69. `developer-sdk` must update this matrix when `developer-sdk.yaml` changes.
70. `developer-sdk` must update this matrix when `developer-sdk-events.yaml` changes.

## Checkpoint

- Authored for `developer-sdk` on 2026-05-20.
- Source contracts checked: `developer-sdk.yaml`, `oya-ecosystem.yaml`, `developer-sdk-events.yaml`, and developer-sdk proto.
- Source policies checked: `developer-scope.cedar`, `admin-scope.cedar`, `payout-scope.cedar`, `public-read.cedar`.
- No in-flight microservice directories were edited.
- Oya VCS scope: `microservices`.
