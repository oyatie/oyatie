---
doc_class: CrossMicroserviceIntegrationTest
scenario_id: CMIT-005-workflow-execution-with-saga
microservices_under_test:
  - workflow-engine
  - workflow-studio
  - payments
  - messenger
status: draft-canonical
date: 2026-05-20
owner: codex-cross-msvc-integration-tests-w1
related_oyatie_adrs:
  - ADR-0035-workflow-engine-state-machine-and-dag-hybrid
  - ADR-0113-vcs-orchestrator-end-to-end
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0249-workflow-engine-universal-orchestrator
  - ADR-0263-observability-emission-contract
---

# Workflow Execution With Saga

## Scenario Description

Leah Martinez, operations director for `tenant-sierra-events-us`, publishes a vendor-onboarding workflow in Workflow Studio that collects a deposit through Payments, sends Messenger updates to a venue coordinator, and compensates by refunding the deposit plus notifying the channel when the final vendor compliance step fails.

## Pre-conditions

### Named tenant fixtures

- Tenant: `tenant-sierra-events-us`.
- Workflow author: `principal-leah-martinez-ops`.
- Venue coordinator: `principal-omar-nasser-venue`.
- Vendor principal: `principal-ivy-chen-vendor`.
- Workflow studio draft: `studio-draft-vendor-onboarding-saga-001`.
- Published workflow id: `workflow-vendor-onboarding-saga`.
- Published workflow version: `v2026-05-20.1`.
- Workflow execution id: `wfexec-sierra-vendor-ivy-001`.
- Deposit payment intent: `payint-sierra-vendor-deposit-001`.
- Refund id: `refund-sierra-vendor-deposit-001`.
- Messenger channel id: `channel-sierra-vendor-onboarding`.
- Failing compliance task id: `task-vendor-insurance-proof`.
- Trace id: `trace-cmit-005-workflow-saga`.
- Idempotency key: `idem-cmit-005-workflow-start`.

### Named Cedar permits

- `permit-workflow-studio-publish-tenant-workflow`.
- `permit-workflow-engine-start-saga`.
- `permit-payments-authorize-vendor-deposit`.
- `permit-payments-refund-saga-compensation`.
- `permit-messenger-send-workflow-status`.
- `permit-workflow-engine-record-compensation`.
- `forbid-workflow-bypass-studio-version-pin`.
- `forbid-payment-capture-after-compensation`.
- `permit-audit-seal-workflow-saga`.

### Named pack activations

- `pack-SOC2-Type-II-baseline`.
- `pack-PCI-DSS-v4`.
- `pack-US-state-privacy-baseline`.
- `pack-workflow-saga-durability`.
- `pack-messenger-work-tenant-retention`.

### Starting state checks

- Workflow Studio draft exists and is editable by Leah.
- Workflow Engine has no execution with id `wfexec-sierra-vendor-ivy-001`.
- Payments sandbox has vendor deposit payment method token `pm-ivy-vendor-ach`.
- Messenger channel exists and includes Leah, Omar, and Ivy.
- Compliance task fixture is configured to fail with `INSURANCE_CERT_EXPIRED`.
- No refund exists for `payint-sierra-vendor-deposit-001`.
- Active workflow version registry has no `v2026-05-20.1` yet.
- Audit stream `tenant-sierra-events-us.workflow` exists.

## Test Steps

1. Validate Workflow Studio graph.
   - API call: `POST /workflow-studio/v1/drafts/studio-draft-vendor-onboarding-saga-001:validate`.
   - Expected response: `200 OK` with `valid=true` and `saga_steps=6`.
   - Assertion: graph includes explicit compensation edge for deposit authorization.

2. Publish the workflow version.
   - API call: `POST /workflow-studio/v1/drafts/studio-draft-vendor-onboarding-saga-001:publish`.
   - Expected response: `201 Created` with workflow id and version `v2026-05-20.1`.
   - Assertion: published definition is immutable and signed by Leah's tenant-scoped principal.

3. Register workflow version in Workflow Engine.
   - API call: `POST /workflow-engine/v1/workflows/workflow-vendor-onboarding-saga/versions`.
   - Expected response: `201 Created` with `state="ACTIVE"`.
   - Assertion: engine stores the exact Studio content hash and rejects local rewrite.

4. Attempt to start unpinned draft version.
   - API call: `POST /workflow-engine/v1/executions` with `workflow_version="draft"`.
   - Expected response: `403 Forbidden` with `error.code="VERSION_PIN_REQUIRED"`.
   - Assertion: denial cites `forbid-workflow-bypass-studio-version-pin`.

5. Start the pinned saga execution.
   - API call: `POST /workflow-engine/v1/executions` with `workflow_version="v2026-05-20.1"`.
   - Expected response: `201 Created` with `execution_id="wfexec-sierra-vendor-ivy-001"` and `state="RUNNING"`.
   - Assertion: execution history starts with `WorkflowExecutionStarted`.

6. Send initial Messenger status.
   - API call: `POST /messenger/v1/channels/channel-sierra-vendor-onboarding/messages`.
   - Expected response: `202 Accepted` with `message_id="msg-saga-started-001"`.
   - Assertion: message has workflow execution id and work-tenant retention label.

7. Create deposit payment intent.
   - API call: `POST /payments/v1/payment-intents`.
   - Expected response: `201 Created` with `payment_intent_id="payint-sierra-vendor-deposit-001"` and `state="REQUIRES_CONFIRMATION"`.
   - Assertion: payment intent references workflow execution id and saga step id.

8. Confirm deposit authorization.
   - API call: `POST /payments/v1/payment-intents/payint-sierra-vendor-deposit-001:confirm`.
   - Expected response: `200 OK` with `state="AUTHORIZED"`.
   - Assertion: workflow engine receives `payments.payment_authorized.v1`.

9. Advance workflow after authorization.
   - API call: `POST /workflow-engine/v1/executions/wfexec-sierra-vendor-ivy-001/signals`.
   - Expected response: `202 Accepted` with signal `DepositAuthorized`.
   - Assertion: execution history records external signal id exactly once.

10. Send deposit received message.
    - API call: `POST /messenger/v1/channels/channel-sierra-vendor-onboarding/messages`.
    - Expected response: `202 Accepted` with `message_id="msg-deposit-authorized-001"`.
    - Assertion: message references payment intent but not raw payment instrument.

11. Run vendor document collection step.
    - API call: `POST /workflow-engine/v1/executions/wfexec-sierra-vendor-ivy-001/tasks/task-vendor-doc-collection:complete`.
    - Expected response: `200 OK` with `next_task="task-vendor-insurance-proof"`.
    - Assertion: task transition is durable in execution history.

12. Run failing insurance proof step.
    - API call: `POST /workflow-engine/v1/executions/wfexec-sierra-vendor-ivy-001/tasks/task-vendor-insurance-proof:complete`.
    - Expected response: `422 Unprocessable Entity` with `error.code="INSURANCE_CERT_EXPIRED"`.
    - Assertion: workflow state becomes `COMPENSATING`.

13. Create compensation refund.
    - API call: `POST /payments/v1/refunds`.
    - Expected response: `201 Created` with `refund_id="refund-sierra-vendor-deposit-001"` and `state="PROCESSING"`.
    - Assertion: refund references original payment intent and compensation step id.

14. Verify capture is forbidden after compensation starts.
    - API call: `POST /payments/v1/payment-intents/payint-sierra-vendor-deposit-001:capture`.
    - Expected response: `409 Conflict` with `error.code="SAGA_COMPENSATION_STARTED"`.
    - Assertion: denial cites `forbid-payment-capture-after-compensation`.

15. Record refund signal in Workflow Engine.
    - API call: `POST /workflow-engine/v1/executions/wfexec-sierra-vendor-ivy-001/signals`.
    - Expected response: `202 Accepted` with signal `DepositRefundInitiated`.
    - Assertion: duplicate refund signal with same idempotency key is ignored.

16. Send compensation Messenger update.
    - API call: `POST /messenger/v1/channels/channel-sierra-vendor-onboarding/messages`.
    - Expected response: `202 Accepted` with `message_id="msg-saga-compensation-001"`.
    - Assertion: message body includes refund id and failing task code.

17. Mark refund settled in sandbox.
    - API call: `POST /payments/v1/refunds/refund-sierra-vendor-deposit-001:settleSandbox`.
    - Expected response: `200 OK` with `state="SUCCEEDED"`.
    - Assertion: payments emits `payments.refund.succeeded.v1`.

18. Complete saga compensation.
    - API call: `POST /workflow-engine/v1/executions/wfexec-sierra-vendor-ivy-001/signals` with `RefundSucceeded`.
    - Expected response: `202 Accepted` with workflow state `COMPENSATED`.
    - Assertion: engine records terminal state `FAILED_COMPENSATED`.

19. Seal workflow saga audit event.
    - API call: `POST /audit-chain/v1/streams/tenant-sierra-events-us.workflow/events`.
    - Expected response: `201 Created` with `event_class="WorkflowSagaCompensated"`.
    - Assertion: audit event includes content hash of Studio workflow version.

20. Read execution history.
    - API call: `GET /workflow-engine/v1/executions/wfexec-sierra-vendor-ivy-001/history`.
    - Expected response: `200 OK` with ordered events from start through compensation.
    - Assertion: no event appears out of order and no compensation edge is missing.

21. Read Messenger channel transcript.
    - API call: `GET /messenger/v1/channels/channel-sierra-vendor-onboarding/messages?trace_id=trace-cmit-005-workflow-saga`.
    - Expected response: `200 OK` with start, deposit, and compensation messages.
    - Assertion: every message links to workflow execution id.

22. Read payment ledger.
    - API call: `GET /payments/v1/payment-intents/payint-sierra-vendor-deposit-001/ledger`.
    - Expected response: `200 OK` with authorization and refund entries, no capture entry.
    - Assertion: total captured amount is zero and total refunded amount equals authorized amount.

23. Replay saga trace.
    - API call: `GET /audit-chain/v1/streams/tenant-sierra-events-us.workflow/events?trace_id=trace-cmit-005-workflow-saga`.
    - Expected response: `200 OK` with workflow, payment, and messenger proof links.
    - Assertion: audit trace reconstructs the failed task and compensation path.

24. Verify final status summary.
    - API call: `GET /workflow-engine/v1/executions/wfexec-sierra-vendor-ivy-001/status-summary`.
    - Expected response: `200 OK` with `terminal_state="FAILED_COMPENSATED"`.
    - Assertion: final summary is impossible unless refund succeeded and compensation message was sent.

## Test Data Fixtures

### Fixture `WorkflowStudioDraftFixture`

```json
{
  "draft_id": "studio-draft-vendor-onboarding-saga-001",
  "tenant_id": "tenant-sierra-events-us",
  "workflow_id": "workflow-vendor-onboarding-saga",
  "author_principal_id": "principal-leah-martinez-ops",
  "nodes": [
    "start",
    "send_initial_message",
    "authorize_deposit",
    "collect_documents",
    "verify_insurance",
    "activate_vendor",
    "compensate_refund_deposit",
    "send_compensation_message"
  ],
  "content_hash": "sha256:studio-vendor-onboarding-saga-v1"
}
```

### Fixture `SagaDefinitionFixture`

```yaml
workflow_id: workflow-vendor-onboarding-saga
version: v2026-05-20.1
saga_steps:
  - step_id: send_initial_message
    service: messenger
    compensation: none
  - step_id: authorize_deposit
    service: payments
    compensation: refund_deposit
  - step_id: collect_documents
    service: workflow-engine
    compensation: none
  - step_id: verify_insurance
    service: workflow-engine
    compensation_trigger: INSURANCE_CERT_EXPIRED
  - step_id: refund_deposit
    service: payments
  - step_id: send_compensation_message
    service: messenger
```

### Fixture `PaymentDepositFixture`

```json
{
  "payment_intent_id": "payint-sierra-vendor-deposit-001",
  "tenant_id": "tenant-sierra-events-us",
  "workflow_execution_id": "wfexec-sierra-vendor-ivy-001",
  "amount_minor": 50000,
  "currency": "USD",
  "payment_method_token": "pm-ivy-vendor-ach",
  "capture_method": "MANUAL_AFTER_VENDOR_APPROVAL"
}
```

### Fixture `MessengerFixture`

```json
{
  "channel_id": "channel-sierra-vendor-onboarding",
  "tenant_id": "tenant-sierra-events-us",
  "members": [
    "principal-leah-martinez-ops",
    "principal-omar-nasser-venue",
    "principal-ivy-chen-vendor"
  ],
  "retention_label": "WORK_TENANT_OPERATIONAL_2Y",
  "message_ids": [
    "msg-saga-started-001",
    "msg-deposit-authorized-001",
    "msg-saga-compensation-001"
  ]
}
```

### Fixture `FailureInjectionFixture`

```yaml
execution_id: wfexec-sierra-vendor-ivy-001
failing_task_id: task-vendor-insurance-proof
failure_code: INSURANCE_CERT_EXPIRED
expected_workflow_state_after_failure: COMPENSATING
expected_terminal_state: FAILED_COMPENSATED
must_not_capture_payment: true
```

### Fixture `ExpectedHistoryFixture`

```yaml
history:
  - WorkflowExecutionStarted
  - MessengerInitialStatusSent
  - PaymentIntentCreated
  - PaymentAuthorized
  - DepositAuthorizedSignalReceived
  - MessengerDepositStatusSent
  - VendorDocumentsCollected
  - InsuranceProofFailed
  - SagaCompensationStarted
  - PaymentRefundCreated
  - PaymentCaptureForbidden
  - DepositRefundSignalReceived
  - MessengerCompensationStatusSent
  - PaymentRefundSucceeded
  - SagaCompensationCompleted
  - WorkflowTerminalFailedCompensated
```

## Assertion Catalogue

### What passes

- `PASS-STUDIO-001`: draft graph validates with explicit compensation edge.
- `PASS-STUDIO-002`: published workflow version is immutable.
- `PASS-ENGINE-001`: engine stores exact Studio content hash.
- `PASS-ENGINE-002`: unpinned draft execution is forbidden.
- `PASS-ENGINE-003`: pinned execution starts once.
- `PASS-ENGINE-004`: failing task moves execution to `COMPENSATING`.
- `PASS-ENGINE-005`: terminal state is `FAILED_COMPENSATED`.
- `PASS-PAYMENTS-001`: deposit authorization records saga step id.
- `PASS-PAYMENTS-002`: refund references original payment intent.
- `PASS-PAYMENTS-003`: capture is forbidden after compensation starts.
- `PASS-PAYMENTS-004`: ledger has no capture entry.
- `PASS-MESSENGER-001`: start message is sent.
- `PASS-MESSENGER-002`: deposit authorized message is sent.
- `PASS-MESSENGER-003`: compensation message is sent.
- `PASS-MESSENGER-004`: every message references execution id.
- `PASS-AUDIT-001`: saga compensation event is sealed.
- `PASS-AUDIT-002`: audit trace links workflow, payments, and messenger.
- `PASS-IDEMPOTENCY-001`: duplicate start and duplicate signal are ignored.
- `PASS-SLO-001`: engine history read fits latency budget.
- `PASS-SLO-002`: refund creation fits compensation budget.

### What fails

- `FAIL-STUDIO-001`: graph publishes without compensation edge.
- `FAIL-ENGINE-001`: engine accepts draft execution.
- `FAIL-ENGINE-002`: engine content hash differs from Studio.
- `FAIL-ENGINE-003`: execution reaches success after failing insurance task.
- `FAIL-PAYMENTS-001`: payment is captured after compensation starts.
- `FAIL-PAYMENTS-002`: refund lacks original payment reference.
- `FAIL-PAYMENTS-003`: ledger has capture entry.
- `FAIL-MESSENGER-001`: compensation message not sent.
- `FAIL-MESSENGER-002`: message omits execution id.
- `FAIL-AUDIT-001`: terminal compensation not sealed.
- `FAIL-AUDIT-002`: audit trace lacks payment proof link.
- `FAIL-IDEMPOTENCY-001`: duplicate execution starts on retry.
- `FAIL-SLO-001`: compensation path exceeds budget.
- `FAIL-CEDAR-001`: mutation lacks Cedar decision.
- `FAIL-STATUS-001`: summary says compensated before refund success.

## Failure Mode Coverage

- `FM-SAGA-001`: Workflow Studio publishes graph without compensation.
- `FM-SAGA-002`: Workflow Engine runs a mutable draft.
- `FM-SAGA-003`: content hash drift between Studio and Engine.
- `FM-SAGA-004`: duplicate execution on idempotent start retry.
- `FM-SAGA-005`: payment authorization missing execution id.
- `FM-SAGA-006`: messenger status lacks execution id.
- `FM-SAGA-007`: failing task does not trigger compensation.
- `FM-SAGA-008`: refund created for wrong payment intent.
- `FM-SAGA-009`: capture allowed after compensation begins.
- `FM-SAGA-010`: duplicate refund signal advances history twice.
- `FM-SAGA-011`: compensation message is skipped.
- `FM-SAGA-012`: refund succeeds but workflow terminal state not updated.
- `FM-SAGA-013`: execution history order is non-deterministic.
- `FM-SAGA-014`: audit event lacks Studio version hash.
- `FM-SAGA-015`: final summary ignores Messenger side effect.
- `FM-SAGA-016`: final summary ignores Payments refund state.
- `FM-SAGA-017`: work-tenant retention missing on channel messages.
- `FM-SAGA-018`: refund amount differs from authorized amount.
- `FM-SAGA-019`: compensation is not replayable from event history.
- `FM-SAGA-020`: failure code is swallowed by generic workflow error.

## Cross-µservice Handoff Validation

- `HANDOFF-STUDIO-ENGINE-OPENAPI`: Studio publish response matches Engine version registration request.
- `HANDOFF-STUDIO-ENGINE-HASH`: Engine stores Studio `content_hash` exactly.
- `HANDOFF-ENGINE-PAYMENTS-OPENAPI`: payment intent request includes workflow execution id and saga step id.
- `HANDOFF-PAYMENTS-ENGINE-ASYNCAPI`: `payments.payment_authorized.v1` maps to `DepositAuthorized` signal.
- `HANDOFF-ENGINE-MESSENGER-OPENAPI`: Messenger status request includes workflow execution id and channel id.
- `HANDOFF-MESSENGER-ENGINE-ASYNCAPI`: message acceptance event can be recorded in execution history.
- `HANDOFF-ENGINE-PAYMENTS-COMPENSATION`: refund request includes original payment intent and compensation step id.
- `HANDOFF-PAYMENTS-ENGINE-REFUND`: `payments.refund.succeeded.v1` maps to compensation completion.
- `HANDOFF-ENGINE-AUDIT-PROTO`: `WorkflowSagaCompensated` includes terminal state and content hash.
- `HANDOFF-PAYMENTS-AUDIT-PROTO`: payment ledger events include authorization, refund, and no capture.
- `HANDOFF-MESSENGER-AUDIT-PROTO`: message events carry work-tenant retention label.
- `HANDOFF-CEDAR-ALL`: every mutation persists a Cedar decision id.
- `HANDOFF-IDEMPOTENCY`: start and signal requests are idempotent.
- `HANDOFF-TRACE`: `trace-cmit-005-workflow-saga` spans Studio, Engine, Payments, Messenger, and Audit.
- `HANDOFF-REPLAY`: execution history plus audit replay reconstruct the same terminal state.

## SLO Conformance

- `SLO-STUDIO-VALIDATE-P95`: Studio graph validation P95 <= 500 ms.
- `SLO-STUDIO-PUBLISH-P95`: workflow publish P95 <= 700 ms.
- `SLO-ENGINE-START-P95`: workflow execution start P95 <= 350 ms.
- `SLO-MESSENGER-SEND-P95`: workflow status message P95 <= 300 ms.
- `SLO-PAYMENT-AUTH-P95`: deposit authorization P95 <= 900 ms.
- `SLO-TASK-TRANSITION-P95`: engine task transition P95 <= 250 ms.
- `SLO-REFUND-CREATE-P95`: compensation refund P95 <= 800 ms.
- `SLO-HISTORY-READ-P95`: execution history read P95 <= 400 ms.
- `SLO-AUDIT-APPEND-P99`: saga audit append P99 <= 150 ms.
- `SLO-COMPENSATION-END-P95`: compensation path P95 <= 3 seconds after injected failure.
- `SLO-AVAILABILITY`: all four service endpoints target 99.95 percent monthly availability.
- `SLO-THROUGHPUT`: one tenant supports 50 concurrent saga executions without history collision.

## Reproducibility

- Named cargo test invocation: `cargo test -p oya-cross-msvc-tests workflow_execution_with_saga -- --ignored --exact --test-threads=1`.
- Named deterministic seed: `CMIT-SEED-2026-05-20-WORKFLOW-SAGA-SIERRA`.
- Required fixture bundle: `fixtures/cross-msvc/workflow-saga-sierra-events.yaml`.
- Required policy bundle: `cedar-bundle-2026-05-20-cross-msvc`.
- Required payment mode: deterministic sandbox with forced insurance failure.
- Required message mode: deterministic Messenger channel fixture.
- Required clock: `2026-05-20T14:00:00Z`.
- Test isolation: execution id and payment intent id are unique per run.
- Stop condition: terminal state is `FAILED_COMPENSATED`, refund succeeded, and compensation message exists.

## References

- `docs/decisions/ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md`.
- `docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md`.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md`.
- `docs/decisions/ADR-0243-cedar-as-universal-gate.md`.
- `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`.
- `docs/decisions/ADR-0263-observability-emission-contract.md`.
- `docs/standards/documentation-rigor.md` section 3.2.5 rows 3, 25, 28.
- `microservices/workflow-engine/contracts/openapi-v1.yaml`.
- `microservices/workflow-studio/contracts/openapi-v1.yaml`.
- `microservices/payments/contracts/openapi-v1.yaml`.
- `microservices/messenger/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/asyncapi-v1.yaml`.
