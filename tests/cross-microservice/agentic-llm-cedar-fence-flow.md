---
doc_class: CrossMicroserviceIntegrationTest
scenario_id: CMIT-003-agentic-llm-cedar-fence-flow
microservices_under_test:
  - intelligence
  - ontology
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
  - ADR-0255-intelligence-as-two-layer-ai-substrate
  - ADR-0257-ontology-read-path
  - ADR-0263-observability-emission-contract
  - ADR-0305-delegated-agent-authority-chain
---

# Agentic LLM Cedar Fence Flow

## Scenario Description

Iris Kwan, legal operations director for `tenant-caldera-legal-eu`, asks an LLM agent named `agent-iris-retention-copilot` to classify a contract folder and propose a retention-label update; the agent may read ontology context and draft a mutation, but governance must Cedar-fence the final action and audit-chain must seal both the proposed action and the decision outcome.

## Pre-conditions

### Named tenant fixtures

- Work tenant: `tenant-caldera-legal-eu`.
- Human delegator: `principal-iris-kwan-legalops`.
- Delegated agent principal: `agent-iris-retention-copilot`.
- Agent token: `delegated-agent-token-cmit-003`.
- Ontology graph namespace: `ontology-caldera-legal-matter-kg`.
- Target object type: `LegalMatterFolder`.
- Target object id: `lmf-caldera-contracts-2026-q2`.
- Existing retention label: `RETENTION_STANDARD_3Y`.
- Proposed retention label: `RETENTION_LEGAL_HOLD_7Y`.
- Matter id: `matter-caldera-vendor-dispute-17`.
- Intelligence model route: `model-route-enterprise-llm-eu-safe`.
- Guardrail profile: `guardrail-legal-retention-eu-v2`.
- Governance policy bundle: `cedar-bundle-legal-retention-2026-05-20`.
- Audit trace id: `trace-cmit-003-agentic-retention`.
- Idempotency key: `idem-cmit-003-agentic-action`.

### Named Cedar permits

- `permit-intelligence-agent-context-read`.
- `permit-ontology-read-legalmatterfolder`.
- `permit-governance-evaluate-agentic-action`.
- `permit-agent-draft-retention-update`.
- `permit-human-approve-retention-update`.
- `forbid-agent-direct-retention-mutation`.
- `forbid-agent-outside-delegation-scope`.
- `permit-audit-seal-agentic-proposal`.
- `permit-audit-seal-policy-decision`.

### Named pack activations

- `pack-GDPR`.
- `pack-EU-AI-Act`.
- `pack-SOC2-Type-II-baseline`.
- `pack-legal-hold-eu`.
- `pack-agent-delegation-transparency`.
- `pack-ADR-0305-delegated-agent-authority-chain`.

### Starting state checks

- Human principal has membership in `tenant-caldera-legal-eu`.
- Agent principal has no standalone tenant membership.
- Agent token is delegation-bound to Iris, purpose `LEGAL_RETENTION_REVIEW`, and expires in 30 minutes.
- Ontology object `lmf-caldera-contracts-2026-q2` exists and belongs to the tenant.
- Governance has `forbid-agent-direct-retention-mutation` active.
- Audit stream `tenant-caldera-legal-eu.agentic-actions` exists.
- Intelligence prompt logging is configured for metadata-only retention.
- No prior action exists for idempotency key `idem-cmit-003-agentic-action`.

## Test Steps

1. Mint a delegated agent session.
   - API call: `POST /identity/v1/delegated-agent-sessions` with `delegated-agent-token-cmit-003`.
   - Expected response: `201 Created` with `agent_principal_id="agent-iris-retention-copilot"`.
   - Assertion: session contains delegator `principal-iris-kwan-legalops` and purpose `LEGAL_RETENTION_REVIEW`.

2. Request ontology context for the target folder.
   - API call: `POST /ontology/v1/query` with object id `lmf-caldera-contracts-2026-q2`.
   - Expected response: `200 OK` with object type `LegalMatterFolder` and active matter relation.
   - Assertion: ontology read records `ontology_read_mode="library_first_network_opt_in"`.

3. Verify agent cannot read unrelated ontology object.
   - API call: `POST /ontology/v1/query` with object id `lmf-caldera-hr-investigation-999`.
   - Expected response: `403 Forbidden` with `error.code="CEDAR_FORBID"`.
   - Assertion: denial cites `forbid-agent-outside-delegation-scope`.

4. Ask intelligence for a retention recommendation.
   - API call: `POST /intelligence/v1/agent-runs`.
   - Expected response: `202 Accepted` with `agent_run_id="run-cmit-003-retention-001"`.
   - Assertion: run request includes model route, guardrail profile, tenant id, and delegation id.

5. Fetch the model draft result.
   - API call: `GET /intelligence/v1/agent-runs/run-cmit-003-retention-001`.
   - Expected response: `200 OK` with `state="COMPLETED"` and proposed label `RETENTION_LEGAL_HOLD_7Y`.
   - Assertion: output is marked `DRAFT_ACTION_ONLY`.

6. Seal the model proposal before evaluation.
   - API call: `POST /audit-chain/v1/streams/tenant-caldera-legal-eu.agentic-actions/events`.
   - Expected response: `201 Created` with `event_class="AgenticActionProposed"`.
   - Assertion: event includes prompt hash, output hash, model route, and delegated agent id.

7. Submit draft action to governance.
   - API call: `POST /governance/v1/agentic-actions:evaluate`.
   - Expected response: `200 OK` with `decision="REQUIRES_HUMAN_APPROVAL"`.
   - Assertion: governance refuses direct mutation by the agent and names `forbid-agent-direct-retention-mutation`.

8. Attempt direct mutation with the agent token.
   - API call: `PATCH /ontology/v1/objects/lmf-caldera-contracts-2026-q2` setting retention label.
   - Expected response: `403 Forbidden` with `error.code="AGENT_DIRECT_MUTATION_FORBIDDEN"`.
   - Assertion: ontology does not change and audit-chain records denial.

9. Fetch human approval task.
   - API call: `GET /governance/v1/approvals?trace_id=trace-cmit-003-agentic-retention`.
   - Expected response: `200 OK` with task `approval-cmit-003-iris-retention`.
   - Assertion: task includes model explanation, policy reason, and explicit approval scope.

10. Human approves bounded action.
    - API call: `POST /governance/v1/approvals/approval-cmit-003-iris-retention:approve`.
    - Expected response: `200 OK` with `decision="ALLOW"` and `cedar_decision_id="cedar-cmit-003-human-allow"`.
    - Assertion: approval principal is Iris, not the delegated agent.

11. Seal policy decision.
    - API call: `POST /audit-chain/v1/streams/tenant-caldera-legal-eu.agentic-actions/events`.
    - Expected response: `201 Created` with `event_class="AgenticActionCedarDecision"`.
    - Assertion: sealed payload links proposal event hash and human approval decision id.

12. Apply retention update through ontology.
    - API call: `PATCH /ontology/v1/objects/lmf-caldera-contracts-2026-q2` with approved decision id.
    - Expected response: `200 OK` with `retention_label="RETENTION_LEGAL_HOLD_7Y"`.
    - Assertion: mutation is accepted only when `cedar_decision_id` equals the human-approved decision.

13. Verify ontology emits mutation event.
    - API call: `GET /eventing/v1/replay?topic=ontology.object.updated.v1&trace_id=trace-cmit-003-agentic-retention`.
    - Expected response: `200 OK` with one `LegalMatterFolderRetentionUpdated` event.
    - Assertion: event actor is the human approval chain, with delegated agent as recommender.

14. Read final object state.
    - API call: `GET /ontology/v1/objects/lmf-caldera-contracts-2026-q2`.
    - Expected response: `200 OK` with legal hold relation and updated retention label.
    - Assertion: object version increments by exactly one.

15. Verify intelligence output cannot be reused for another object.
    - API call: `POST /governance/v1/agentic-actions:evaluate` reusing proposal hash against `lmf-caldera-contracts-2025-q4`.
    - Expected response: `409 Conflict` with `error.code="PROPOSAL_RESOURCE_MISMATCH"`.
    - Assertion: governance binds proposal hash to original ontology object id.

16. Verify expired agent token is rejected.
    - API call: `POST /intelligence/v1/agent-runs` using simulated expired token.
    - Expected response: `401 Unauthorized` with `error.code="DELEGATION_EXPIRED"`.
    - Assertion: no model call is made and no ontology read occurs.

17. Query audit trace.
    - API call: `GET /audit-chain/v1/streams/tenant-caldera-legal-eu.agentic-actions/events?trace_id=trace-cmit-003-agentic-retention`.
    - Expected response: `200 OK` with proposal, denial, approval, decision, and mutation events.
    - Assertion: every event has Merkle proof and immutable actor chain.

18. Verify EU AI Act transparency record.
    - API call: `GET /governance/v1/ai-act/transparency-records/run-cmit-003-retention-001`.
    - Expected response: `200 OK` with model route, purpose, human approver, and risk tier.
    - Assertion: transparency record omits raw privileged document text.

19. Re-run the same approved mutation idempotently.
    - API call: `PATCH /ontology/v1/objects/lmf-caldera-contracts-2026-q2` with same idempotency key.
    - Expected response: `200 OK` with unchanged object version after first mutation.
    - Assertion: idempotency prevents duplicate retention update events.

20. Reconstruct decision graph.
    - API call: `GET /governance/v1/agentic-actions/trace-cmit-003-agentic-retention/decision-graph`.
    - Expected response: `200 OK` with nodes for prompt, ontology context, proposal, Cedar decision, approval, and mutation.
    - Assertion: graph is complete enough to explain why the mutation happened.

21. Validate audit-to-ontology consistency.
    - API call: `POST /audit-chain/v1/consistency-checks/ontology-mutation`.
    - Expected response: `200 OK` with `consistent=true`.
    - Assertion: audit event hash matches ontology object version hash.

22. Verify final scenario status.
    - API call: `GET /intelligence/v1/agent-runs/run-cmit-003-retention-001/finalization`.
    - Expected response: `200 OK` with `final_state="HUMAN_APPROVED_ACTION_APPLIED"`.
    - Assertion: final state is not possible without governance approval and audit proof.

## Test Data Fixtures

### Fixture `DelegatedAgentSessionFixture`

```json
{
  "agent_principal_id": "agent-iris-retention-copilot",
  "delegator_principal_id": "principal-iris-kwan-legalops",
  "tenant_id": "tenant-caldera-legal-eu",
  "purpose": "LEGAL_RETENTION_REVIEW",
  "scope": {
    "object_type": "LegalMatterFolder",
    "object_ids": [
      "lmf-caldera-contracts-2026-q2"
    ],
    "allowed_actions": [
      "READ_CONTEXT",
      "DRAFT_ACTION"
    ]
  },
  "expires_at": "2026-05-20T14:30:00Z"
}
```

### Fixture `OntologyContextFixture`

```json
{
  "namespace": "ontology-caldera-legal-matter-kg",
  "object_type": "LegalMatterFolder",
  "object_id": "lmf-caldera-contracts-2026-q2",
  "tenant_id": "tenant-caldera-legal-eu",
  "properties": {
    "current_retention_label": "RETENTION_STANDARD_3Y",
    "matter_id": "matter-caldera-vendor-dispute-17",
    "contains_executed_contracts": true,
    "litigation_hold_signal": true
  },
  "relations": [
    {
      "type": "belongs_to_matter",
      "target": "matter-caldera-vendor-dispute-17"
    }
  ]
}
```

### Fixture `IntelligenceRunFixture`

```yaml
agent_run_id: run-cmit-003-retention-001
tenant_id: tenant-caldera-legal-eu
model_route: model-route-enterprise-llm-eu-safe
guardrail_profile: guardrail-legal-retention-eu-v2
prompt_hash: sha256:prompt-cmit-003
context_hash: sha256:ontology-context-cmit-003
output:
  proposed_action: update_retention_label
  target_object_id: lmf-caldera-contracts-2026-q2
  proposed_label: RETENTION_LEGAL_HOLD_7Y
  confidence: 0.91
  mode: DRAFT_ACTION_ONLY
```

### Fixture `GovernanceDecisionFixture`

```json
{
  "action_id": "agentic-action-cmit-003",
  "tenant_id": "tenant-caldera-legal-eu",
  "requested_by": "agent-iris-retention-copilot",
  "delegated_by": "principal-iris-kwan-legalops",
  "resource": "ontology://LegalMatterFolder/lmf-caldera-contracts-2026-q2",
  "action": "UpdateRetentionLabel",
  "initial_decision": "REQUIRES_HUMAN_APPROVAL",
  "final_decision": "ALLOW",
  "final_decision_id": "cedar-cmit-003-human-allow"
}
```

### Fixture `AuditEventFixture`

```yaml
stream: tenant-caldera-legal-eu.agentic-actions
trace_id: trace-cmit-003-agentic-retention
events:
  - AgenticActionProposed
  - AgenticActionDirectMutationDenied
  - AgenticActionHumanApprovalRequested
  - AgenticActionHumanApproved
  - AgenticActionCedarDecision
  - OntologyObjectMutationApplied
  - AiActTransparencyRecordWritten
```

### Fixture `NegativeReuseFixture`

```json
{
  "original_proposal_hash": "sha256:proposal-cmit-003-retention-q2",
  "original_object_id": "lmf-caldera-contracts-2026-q2",
  "attempted_object_id": "lmf-caldera-contracts-2025-q4",
  "expected_status": 409,
  "expected_error": "PROPOSAL_RESOURCE_MISMATCH"
}
```

## Assertion Catalogue

### What passes

- `PASS-DELEGATION-001`: delegated agent session is purpose-bound.
- `PASS-DELEGATION-002`: session cannot outlive token expiry.
- `PASS-ONTOLOGY-001`: ontology context is scoped to the allowed object.
- `PASS-ONTOLOGY-002`: unrelated object read is forbidden.
- `PASS-INTEL-001`: model output is draft-only.
- `PASS-INTEL-002`: model route and guardrail profile are recorded.
- `PASS-GOV-001`: direct agent mutation requires human approval.
- `PASS-GOV-002`: approved decision id is bound to one resource.
- `PASS-GOV-003`: proposal hash cannot be replayed against another object.
- `PASS-AUDIT-001`: proposal hash is sealed before policy decision.
- `PASS-AUDIT-002`: Cedar decision is sealed before mutation.
- `PASS-AUDIT-003`: mutation audit hash matches ontology object version.
- `PASS-AI-ACT-001`: transparency record includes model route and human approver.
- `PASS-AI-ACT-002`: transparency record excludes raw privileged document text.
- `PASS-MUTATION-001`: ontology version increments by exactly one.
- `PASS-MUTATION-002`: idempotent retry does not duplicate mutation.
- `PASS-TRACE-001`: decision graph is reconstructable.
- `PASS-POLICY-001`: every mutation path carries a Cedar decision id.
- `PASS-SLO-001`: governance evaluation fits policy-decision budget.
- `PASS-SLO-002`: audit consistency check returns within budget.

### What fails

- `FAIL-DELEGATION-001`: agent session is not bound to Iris.
- `FAIL-DELEGATION-002`: expired agent token reaches model gateway.
- `FAIL-ONTOLOGY-001`: agent reads unrelated object.
- `FAIL-INTEL-001`: LLM output is treated as executable command.
- `FAIL-GOV-001`: direct agent mutation succeeds.
- `FAIL-GOV-002`: human approval does not name explicit resource.
- `FAIL-GOV-003`: proposal hash replays against another object.
- `FAIL-AUDIT-001`: model proposal not sealed.
- `FAIL-AUDIT-002`: policy decision not sealed.
- `FAIL-AUDIT-003`: audit event actor chain omits delegator.
- `FAIL-AI-ACT-001`: transparency record missing human approver.
- `FAIL-AI-ACT-002`: transparency record leaks privileged text.
- `FAIL-MUTATION-001`: ontology changes without approved decision id.
- `FAIL-MUTATION-002`: object version increments twice on idempotent retry.
- `FAIL-TRACE-001`: decision graph omits ontology context hash.

## Failure Mode Coverage

- `FM-AGENTIC-001`: LLM agent escapes delegation purpose.
- `FM-AGENTIC-002`: ontology read path ignores tenant id.
- `FM-AGENTIC-003`: ontology read path ignores object allow-list.
- `FM-AGENTIC-004`: model output bypasses governance.
- `FM-AGENTIC-005`: governance treats agent as human approver.
- `FM-AGENTIC-006`: Cedar decision is not resource-bound.
- `FM-AGENTIC-007`: proposal hash replay mutates another object.
- `FM-AGENTIC-008`: audit stream lacks prompt/output hash.
- `FM-AGENTIC-009`: privileged legal text leaks into audit-chain.
- `FM-AGENTIC-010`: EU AI Act transparency record is missing.
- `FM-AGENTIC-011`: expired delegated token still reaches intelligence.
- `FM-AGENTIC-012`: idempotent retry creates duplicate retention events.
- `FM-AGENTIC-013`: object version changes before human approval.
- `FM-AGENTIC-014`: finalization says applied without audit proof.
- `FM-AGENTIC-015`: trace id lost between intelligence and governance.
- `FM-AGENTIC-016`: guardrail profile not recorded.
- `FM-AGENTIC-017`: active policy bundle version drifts mid-flow.
- `FM-AGENTIC-018`: denied direct mutation fails to emit audit event.
- `FM-AGENTIC-019`: decision graph cannot support regulator review.
- `FM-AGENTIC-020`: human approval task omits model explanation.

## Cross-µservice Handoff Validation

- `HANDOFF-IDENTITY-INTELLIGENCE-OPENAPI`: delegated agent session token maps to intelligence `agent_principal_id`.
- `HANDOFF-INTELLIGENCE-ONTOLOGY-OPENAPI`: intelligence context request carries ontology object id and purpose.
- `HANDOFF-ONTOLOGY-INTELLIGENCE-ASYNCAPI`: ontology context hash is returned to intelligence run metadata.
- `HANDOFF-INTELLIGENCE-GOVERNANCE-OPENAPI`: draft action schema matches governance `AgenticActionEvaluateRequest`.
- `HANDOFF-GOVERNANCE-INTELLIGENCE-ASYNCAPI`: governance decision updates intelligence run finalization state.
- `HANDOFF-GOVERNANCE-ONTOLOGY-OPENAPI`: ontology accepts mutation only with `cedar_decision_id`.
- `HANDOFF-ONTOLOGY-AUDIT-PROTO`: ontology mutation event includes object version hash.
- `HANDOFF-INTELLIGENCE-AUDIT-PROTO`: proposal audit event includes model route and output hash.
- `HANDOFF-GOVERNANCE-AUDIT-PROTO`: policy decision event includes initial and final Cedar decisions.
- `HANDOFF-AI-ACT-GOVERNANCE`: transparency record receives human approver and risk tier.
- `HANDOFF-TRACE`: W3C trace context is preserved across intelligence, ontology, governance, and audit-chain.
- `HANDOFF-CEDAR`: Cedar entity names use the same tenant id and object id across services.
- `HANDOFF-IDEMPOTENCY`: idempotency key binds to one ontology mutation.
- `HANDOFF-ERROR`: direct mutation denial maps to `AGENT_DIRECT_MUTATION_FORBIDDEN`.
- `HANDOFF-REPLAY`: audit replay reconstructs the same decision graph as governance.

## SLO Conformance

- `SLO-DELEGATED-SESSION-P95`: delegated session mint P95 <= 200 ms.
- `SLO-ONTOLOGY-QUERY-P95`: scoped ontology query P95 <= 350 ms.
- `SLO-INTELLIGENCE-RUN-P95`: model draft completion P95 <= 8 seconds for seeded deterministic model.
- `SLO-GOV-EVALUATE-P95`: governance action evaluation P95 <= 250 ms.
- `SLO-HUMAN-APPROVAL-API-P95`: approval API P95 <= 300 ms excluding human think time.
- `SLO-ONTOLOGY-MUTATE-P95`: approved mutation P95 <= 400 ms.
- `SLO-AUDIT-APPEND-P99`: audit event append P99 <= 150 ms.
- `SLO-CONSISTENCY-CHECK-P95`: audit-to-ontology consistency check P95 <= 2 seconds.
- `SLO-THROUGHPUT`: one tenant supports 25 concurrent delegated agent draft runs without policy cache collision.
- `SLO-AVAILABILITY`: intelligence, ontology, governance, and audit-chain endpoints target 99.95 percent monthly availability.
- `SLO-PRIVACY`: zero raw privileged legal text in audit-chain or transparency output.
- `SLO-DECISION-GRAPH`: final decision graph returns within 1 second for <= 20 nodes.

## Reproducibility

- Named cargo test invocation: `cargo test -p oya-cross-msvc-tests agentic_llm_cedar_fence_flow -- --ignored --exact --test-threads=1`.
- Named deterministic seed: `CMIT-SEED-2026-05-20-AGENTIC-CEDAR-FENCE`.
- Required fixture bundle: `fixtures/cross-msvc/agentic-retention-caldera.yaml`.
- Required model mode: deterministic stub route `model-route-enterprise-llm-eu-safe`.
- Required policy bundle: `cedar-bundle-legal-retention-2026-05-20`.
- Required clock: `2026-05-20T14:00:00Z`.
- Required audit stream: `tenant-caldera-legal-eu.agentic-actions`.
- Test isolation: ontology object is reset to version baseline after audit proof export.
- Stop condition: mutation applies only after human approval and audit consistency passes.

## References

- `docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md`.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md`.
- `docs/decisions/ADR-0243-cedar-as-universal-gate.md`.
- `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`.
- `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`.
- `docs/decisions/ADR-0257-ontology-read-path.md`.
- `docs/decisions/ADR-0263-observability-emission-contract.md`.
- `docs/standards/documentation-rigor.md` section 3.2.5 row 28.
- `microservices/intelligence/contracts/openapi-v1.yaml`.
- `microservices/ontology/contracts/openapi-v1.yaml`.
- `microservices/governance/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/asyncapi-v1.yaml`.
- `microservices/audit-chain/contracts/audit-event-v1.proto`.
