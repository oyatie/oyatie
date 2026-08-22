---
doc_class: Standard
title: Workflow Substrate Engine Standard
status: Accepted
date: 2026-05-20
owner: axis-workflow + council-architecture
related_oyatie_adrs:
  - ADR-0035
  - ADR-0145
  - ADR-0141
  - ADR-0222
  - ADR-0316
enforced_by:
  - governance-workflow-substrate-engine
  - governance-workflow-vs-direct-grpc
  - governance-saga-compensation
canonical_paths:
  - specs/microservices/workflow.json
  - microservices/workflow-engine/
  - docs/standards/workflow-vs-direct-grpc-rubric.md
  - docs/standards/saga-compensation-policy.md
---

# Workflow Substrate Engine Standard

The workflow substrate is the durable orchestration layer for cross-service,
human-in-the-loop, policy-gated, timed, compensating, and long-running work. It
is not a generic background-job bucket. It is the engine that lets capability
tiers compose actions across ontology, Cedar, audit-chain, and microservice
contracts without creating product-specific service silos.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to `microservices/workflow-engine/`.

It applies to `specs/microservices/workflow.json`.

It applies to workflow templates.

It applies to state-machine definitions.

It applies to DAG definitions.

It applies to approval workflows.

It applies to timers, deadlines, and escalations.

It applies to saga compensation paths.

It applies to cross-service orchestration.

It applies to capability-tier workflow contributions.

It applies to workflow OpenAPI, AsyncAPI, and proto contracts.

It does not cover direct synchronous calls that satisfy the direct-gRPC rubric.

It does not authorize workflow for every two-step operation.

It does not replace domain invariants inside owning services.

## Normative Requirements

W-001. Every workflow template MUST have a stable id.

W-002. Every workflow template MUST have a version.

W-003. Every workflow template MUST declare owning microservice.

W-004. Every workflow template MUST declare capability tier bindings when tenant-visible.

W-005. Every workflow template MUST declare ontology object and action references when applicable.

W-006. Every workflow template MUST declare Cedar policy requirements.

W-007. Every workflow template MUST declare audit event classes.

W-008. Every workflow template MUST declare idempotency key shape.

W-009. Every workflow template MUST declare cancellation behavior.

W-010. Every workflow template MUST declare retry behavior.

W-011. Every workflow template MUST declare timeout behavior.

W-012. Every workflow template MUST declare compensation behavior for state mutation.

W-013. Every workflow template MUST declare human approval steps when required.

W-014. Every workflow template MUST declare tenant and principal context.

W-015. Every workflow template MUST declare data classes.

W-016. Every workflow template MUST declare residency pack overlays.

W-017. Every workflow template MUST declare observability signals.

W-018. Every workflow template MUST declare SLO impact.

W-019. Every workflow template MUST declare cost allocation dimensions.

W-020. Every workflow template MUST declare replay behavior.

W-021. State-machine workflows MUST enumerate all states.

W-022. State-machine workflows MUST enumerate all terminal states.

W-023. State-machine workflows MUST enumerate all transitions.

W-024. State-machine workflows MUST name transition guards.

W-025. State-machine workflows MUST name transition side effects.

W-026. State-machine workflows MUST include an invalid-transition behavior.

W-027. DAG workflows MUST enumerate all nodes.

W-028. DAG workflows MUST enumerate all edges.

W-029. DAG workflows MUST declare fan-out limits.

W-030. DAG workflows MUST declare join semantics.

W-031. DAG workflows MUST declare partial failure handling.

W-032. DAG workflows MUST declare ordering constraints.

W-033. Approval workflows MUST declare approver role, quorum, deadline, and escalation.

W-034. Approval workflows MUST bind approver authority to Cedar.

W-035. Approval workflows MUST emit evidence for approve, reject, timeout, and escalate.

W-036. Timer workflows MUST use monotonic logical deadlines where possible.

W-037. Timer workflows MUST tolerate clock skew per time-coordination standards.

W-038. Cross-service workflows MUST call public contracts only.

W-039. Cross-service workflows MUST NOT call another service database.

W-040. Cross-service workflows MUST NOT bypass owner-service validation.

W-041. Workflow steps MUST be idempotent.

W-042. Workflow steps MUST be replay safe.

W-043. Workflow steps MUST have typed errors.

W-044. Workflow steps MUST classify retryable and non-retryable failures.

W-045. Workflow steps MUST not swallow policy denials.

W-046. Workflow steps MUST include request id propagation.

W-047. Workflow steps MUST include audit correlation ids.

W-048. Workflow steps MUST include tenant scope.

W-049. Workflow steps MUST include principal scope unless system-owned.

W-050. System-owned steps MUST cite the system principal.

W-051. A saga MUST declare forward steps.

W-052. A saga MUST declare compensating steps.

W-053. A saga MUST declare compensation ordering.

W-054. A saga MUST declare compensation failure behavior.

W-055. A saga MUST declare irreversible external side effects.

W-056. A saga MUST declare manual recovery when compensation cannot fully restore state.

W-057. A workflow template MUST not encode UI copy.

W-058. A workflow template MUST not encode product marketing names as service boundaries.

W-059. A workflow template MUST be contract-testable.

W-060. A workflow template MUST be fixture-testable.

## Worked Examples

### Example 1: Approval state machine

```yaml
id: workflow.approval-routing.v1
kind: state_machine
owner_microservice: workflow-engine
states:
  - draft
  - pending_approval
  - approved
  - rejected
  - expired
terminal_states:
  - approved
  - rejected
  - expired
transitions:
  - from: draft
    to: pending_approval
    guard: cedar:workflow.approval.submit
  - from: pending_approval
    to: approved
    guard: cedar:workflow.approval.approve
    audit_event: EVT-WORKFLOW-APPROVAL-APPROVED-V1
```

This passes because states, transitions, guards, and audit events are explicit.

### Example 2: Cross-service onboarding DAG

```yaml
id: workflow.tenant-onboarding.v1
kind: dag
nodes:
  verify_identity:
    service: identity
    contract: identity-verification-v1.openapi.yaml
  create_tenant:
    service: tenancy
    contract: tenant-lifecycle-v1.openapi.yaml
  bind_policy:
    service: policy-engine
    contract: cedar-policy-v1.openapi.yaml
edges:
  - verify_identity -> create_tenant
  - create_tenant -> bind_policy
```

This passes because each node calls a public service contract.

### Example 3: Invalid direct database orchestration

```yaml
step:
  sql: "update tenancy.tenants set status = 'active'"
```

This fails because workflow cannot mutate another service database directly.

### Example 4: Saga compensation

```yaml
forward:
  - reserve_inventory
  - authorize_payment
  - create_shipment
compensate:
  create_shipment: cancel_shipment
  authorize_payment: void_authorization
  reserve_inventory: release_inventory
irreversible:
  - carrier_pickup_confirmed
manual_recovery: microservices/workflow-engine/runbooks/shipment-saga-recovery.md
```

This passes because irreversible points are explicit.

### Example 5: Human approval with Cedar

```yaml
approval:
  role: tenant_admin
  quorum: 2
  deadline: PT24H
  escalation: security_council
  cedar_action: Action::"ApproveHighRiskWorkflow"
```

This passes because authority and deadline are explicit.

## Verification

Primary command:

```bash
oya gate validate workflow-substrate-engine --scope microservices/workflow-engine
```

The checker MUST parse workflow templates.

The checker MUST parse workflow contracts.

The checker MUST parse capability tier records.

The checker MUST parse ontology references.

The checker MUST parse Cedar action references.

The checker MUST parse audit event references.

The checker MUST reject unversioned template ids.

The checker MUST reject missing owners.

The checker MUST reject missing idempotency keys.

The checker MUST reject mutating steps without audit events.

The checker MUST reject cross-service database references.

The checker MUST reject missing compensation on sagas.

The checker MUST reject missing terminal states on state machines.

The checker MUST reject DAG cycles unless explicitly marked as loop constructs.

The checker MUST reject approval steps without Cedar role bindings.

The checker MUST reject timeouts without escalation or terminal behavior.

The checker MUST reject workflow templates without fixture tests.

The checker SHOULD emit graph diagrams as evidence.

The checker SHOULD emit a workflow-to-service call matrix.

The checker SHOULD emit stale template warnings for unused versions.

## Common Anti-Patterns

Using workflow for simple CRUD is an anti-pattern.

Using workflow as a hidden integration bus is an anti-pattern.

Using SQL steps against another service is an anti-pattern.

Using unbounded fan-out is an anti-pattern.

Using retries without idempotency is an anti-pattern.

Using compensation without ordering is an anti-pattern.

Using human approval without Cedar authority is an anti-pattern.

Using timers without skew tolerance is an anti-pattern.

Using workflow templates to store UI text is an anti-pattern.

Using workflow templates as product modules is an anti-pattern.

Using direct gRPC where saga compensation is required is an anti-pattern.

Using workflow where direct gRPC is required for latency is an anti-pattern.

Using untyped step errors is an anti-pattern.

Using generic `failed` terminal state with no recovery class is an anti-pattern.

Using audit events as optional observability is an anti-pattern.

## Cross-References

`docs/decisions/ADR-0700-ci-admission-live-apex.md` binds hybrid workflow shape.

`docs/decisions/ADR-0701-monorepo-capability-live-apex.md` binds workflow versus direct service calls.

`docs/decisions/ADR-0704-k8s-port-live-apex.md` binds saga compensation.

`docs/decisions/ADR-0709-general-live-apex.md` binds capability tiers.

`docs/standards/workflow-vs-direct-grpc-rubric.md` decides workflow versus direct call.

`docs/standards/saga-compensation-policy.md` gives compensation rules.

`docs/standards/ontology-projection-substrate.md` binds ontology action integration.

`docs/standards/cedar-policy-authoring.md` binds policy guards.

`docs/standards/asyncapi-3-1-authoring.md` binds workflow events.

`docs/standards/proto3-authoring.md` binds internal contracts.

## Substance Bar Compliance Checklist

WF-SB-001. Verify workflow template id is stable.

WF-SB-002. Verify workflow template version is present.

WF-SB-003. Verify owning microservice is declared.

WF-SB-004. Verify capability tier bindings are declared.

WF-SB-005. Verify ontology object references resolve.

WF-SB-006. Verify ontology action references resolve.

WF-SB-007. Verify Cedar policy references resolve.

WF-SB-008. Verify audit event classes resolve.

WF-SB-009. Verify idempotency key shape.

WF-SB-010. Verify cancellation behavior.

WF-SB-011. Verify retry behavior.

WF-SB-012. Verify timeout behavior.

WF-SB-013. Verify compensation behavior.

WF-SB-014. Verify human approval steps.

WF-SB-015. Verify tenant context.

WF-SB-016. Verify principal context.

WF-SB-017. Verify data classes.

WF-SB-018. Verify residency pack overlays.

WF-SB-019. Verify observability signals.

WF-SB-020. Verify SLO impact.

WF-SB-021. Verify cost dimensions.

WF-SB-022. Verify replay behavior.

WF-SB-023. Verify state list.

WF-SB-024. Verify terminal states.

WF-SB-025. Verify transitions.

WF-SB-026. Verify transition guards.

WF-SB-027. Verify transition side effects.

WF-SB-028. Verify invalid transition behavior.

WF-SB-029. Verify DAG nodes.

WF-SB-030. Verify DAG edges.

WF-SB-031. Verify fan-out limits.

WF-SB-032. Verify join semantics.

WF-SB-033. Verify partial failure handling.

WF-SB-034. Verify ordering constraints.

WF-SB-035. Verify approver role.

WF-SB-036. Verify approval quorum.

WF-SB-037. Verify approval deadline.

WF-SB-038. Verify approval escalation.

WF-SB-039. Verify timer skew tolerance.

WF-SB-040. Verify public contract calls.

WF-SB-041. Check `workflow.approval-routing.v1`.

WF-SB-042. Check `workflow.tenant-onboarding.v1`.

WF-SB-043. Check `workflow.case-escalation.v1`.

WF-SB-044. Check `workflow.deadline-fire.v1`.

WF-SB-045. Check `workflow.dsar-orchestration.v1`.

WF-SB-046. Check `workflow.cross-tenant-launch.v1`.

WF-SB-047. Check `workflow.shipment-saga.v1`.

WF-SB-048. Check `workflow.audit-investigation.v1`.

WF-SB-049. Check `workflow.capability-tier-grant.v1`.

WF-SB-050. Check `workflow.tenant-offboarding.v1`.

WF-SB-051. Reject SQL steps against service databases.

WF-SB-052. Reject retries without idempotency.

WF-SB-053. Reject saga without compensation.

WF-SB-054. Reject approval without Cedar.

WF-SB-055. Reject timer without terminal behavior.

WF-SB-056. Reject mutating step without audit.

WF-SB-057. Reject untyped step errors.

WF-SB-058. Reject unbounded fan-out.

WF-SB-059. Reject cross-service private contract calls.

WF-SB-060. Reject workflow template UI copy.

WF-SB-061. Emit template count.

WF-SB-062. Emit state machine count.

WF-SB-063. Emit DAG count.

WF-SB-064. Emit saga count.

WF-SB-065. Emit approval count.

WF-SB-066. Emit timer count.

WF-SB-067. Emit cross-service call count.

WF-SB-068. Emit compensation coverage count.

WF-SB-069. Emit policy binding count.

WF-SB-070. Emit audit binding count.

WF-SB-071. Preserve service-owned invariants.

WF-SB-072. Preserve workflow as orchestration only.

WF-SB-073. Preserve domain validation in owner service.

WF-SB-074. Preserve ontology references by id.

WF-SB-075. Preserve Cedar guards before mutation.

WF-SB-076. Preserve audit trail for every state transition.

WF-SB-077. Preserve replay safety for every step.

WF-SB-078. Preserve rollback or compensation.

WF-SB-079. Preserve direct-gRPC rubric for low-latency calls.

WF-SB-080. Preserve workflow engine as substrate, not product silo.

## Extended Worked Example: Regulated Human-Review Workflow

The workflow below is intentionally substrate-shaped: it coordinates timers,
human gates, compensation, Cedar decisions, and audit evidence without becoming
the product owner for the underlying business decision.

```yaml
workflow_id: regulated-human-review-v1
owning_microservice: workflow-engine
business_authority: foundry
trigger_event: foundry.auto_decision.blocked.v1
related_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
states:
  - name: received
    invariant: source event validated and idempotency key stored
  - name: policy_checked
    invariant: Cedar decision attached to workflow context
  - name: human_review_requested
    invariant: review task created in tasks µservice
  - name: waiting_for_human
    invariant: deadline timer registered
  - name: approved
    invariant: approval audit event emitted
  - name: rejected
    invariant: rejection reason emitted
  - name: compensated
    invariant: downstream provisional effects reverted
  - name: completed
    invariant: final event published
timeouts:
  human_review_deadline:
    duration: PT72H
    on_expire: reject_and_compensate
ports:
  task_port: microservices/tasks/contracts/review-task-v1.openapi.yaml
  policy_port: microservices/policy-engine/contracts/decision-v1.proto
  audit_port: microservices/audit/contracts/audit-event-v1.asyncapi.yaml
```

## Extended State Transition Matrix

| ID | From | To | Guard | Side effect | Evidence |
|---|---|---|---|---|---|
| WF-MAT-001 | none | received | idempotency key absent | store workflow row | `EVT-WORKFLOW-RECEIVED` |
| WF-MAT-002 | received | policy_checked | Cedar bundle valid | call policy port | `EVT-WORKFLOW-POLICY-CHECKED` |
| WF-MAT-003 | policy_checked | human_review_requested | decision requires human | create task | `EVT-HUMAN-REVIEW-REQUESTED` |
| WF-MAT-004 | human_review_requested | waiting_for_human | task id stored | register timer | `EVT-WORKFLOW-TIMER-SET` |
| WF-MAT-005 | waiting_for_human | approved | reviewer allowed | publish approval | `EVT-HUMAN-REVIEW-APPROVED` |
| WF-MAT-006 | waiting_for_human | rejected | reviewer denied | publish rejection | `EVT-HUMAN-REVIEW-REJECTED` |
| WF-MAT-007 | waiting_for_human | rejected | timer expired | publish timeout | `EVT-HUMAN-REVIEW-TIMED-OUT` |
| WF-MAT-008 | rejected | compensated | provisional effect exists | call compensation port | `EVT-WORKFLOW-COMPENSATED` |
| WF-MAT-009 | approved | completed | all publishes acked | close workflow | `EVT-WORKFLOW-COMPLETED` |
| WF-MAT-010 | compensated | completed | compensation acked | close workflow | `EVT-WORKFLOW-COMPLETED` |
| WF-MAT-011 | received | rejected | source schema invalid | no external effect | `EVT-WORKFLOW-REJECTED` |
| WF-MAT-012 | policy_checked | rejected | Cedar forbid | no task created | `EVT-WORKFLOW-DENIED` |
| WF-MAT-013 | human_review_requested | compensated | task create failed after provisional write | revert provisional write | `EVT-WORKFLOW-COMPENSATED` |
| WF-MAT-014 | waiting_for_human | waiting_for_human | duplicate reminder | emit reminder only | `EVT-WORKFLOW-REMINDER-SENT` |
| WF-MAT-015 | completed | completed | duplicate completion event | no-op idempotent ack | `EVT-WORKFLOW-DUPLICATE-IGNORED` |
| WF-MAT-016 | rejected | rejected | duplicate rejection | no-op idempotent ack | `EVT-WORKFLOW-DUPLICATE-IGNORED` |
| WF-MAT-017 | approved | approved | duplicate approval | no-op idempotent ack | `EVT-WORKFLOW-DUPLICATE-IGNORED` |
| WF-MAT-018 | any | compensated | compensation policy allows | execute saga step | `EVT-SAGA-COMPENSATION-STEP` |
| WF-MAT-019 | any | rejected | tenant pack denies | emit regulatory reason | `EVT-WORKFLOW-REGULATORY-DENY` |
| WF-MAT-020 | any | completed | terminal invariant true | final snapshot | `EVT-WORKFLOW-FINALIZED` |

## Extended Workflow Verification Matrix

| ID | Concern | Required evidence | Checker |
|---|---|---|---|
| WF-VER-001 | Idempotency | duplicate trigger test | `check-workflow-idempotency` |
| WF-VER-002 | Timers | deterministic timer fixture | `check-workflow-timers` |
| WF-VER-003 | Compensation | saga rollback fixture | `check-saga-compensation` |
| WF-VER-004 | Policy | Cedar guard fixture | `check-workflow-cedar-guards` |
| WF-VER-005 | Audit | transition audit events | `check-audit-emission` |
| WF-VER-006 | Outbox | event publish ack | `check-outbox-pattern` |
| WF-VER-007 | Retry | bounded retry manifest | `check-workflow-retry-bounds` |
| WF-VER-008 | Replay | replay from event log | `check-workflow-replay` |
| WF-VER-009 | Versioning | workflow template version | `check-workflow-versioning` |
| WF-VER-010 | Direct call | direct-gRPC exception absent or justified | `check-direct-grpc-rubric` |
| WF-VER-011 | Human gate | actor authorization evidence | `check-human-gate` |
| WF-VER-012 | Data class | context field labels | `check-data-class` |
| WF-VER-013 | Residency | workflow state cell binding | `check-residency-parity` |
| WF-VER-014 | Pack overlay | regulatory timer override | `check-pack-overlay` |
| WF-VER-015 | Metrics | latency and stuck-workflow metrics | `check-workflow-metrics` |
| WF-VER-016 | Runbook | stuck execution runbook link | `check-runbook-linkage` |
| WF-VER-017 | Test | state-machine transition coverage | `check-workflow-test-coverage` |
| WF-VER-018 | Fixture | invalid transition fixture | `check-workflow-fixtures` |
| WF-VER-019 | Schema | OpenAPI/AsyncAPI/Proto parity | `check-contract-parity` |
| WF-VER-020 | Promote | evidence names workflow template | `oya-vcs-admission` |

## Extended Review Questions

WF-REV-001. Does the workflow cite the business authority service?

WF-REV-002. Does every state have a durable invariant?

WF-REV-003. Does every transition have a guard?

WF-REV-004. Does every side effect happen after idempotency storage?

WF-REV-005. Does every external call go through a named port?

WF-REV-006. Does every timer have an expiry action?

WF-REV-007. Does every retry have a max attempt count?

WF-REV-008. Does every compensation step have a reverse operation or explicit no-op reason?

WF-REV-009. Does every human approval carry actor id and policy decision?

WF-REV-010. Does every terminal state emit audit evidence?

WF-REV-011. Does replay produce the same terminal state?

WF-REV-012. Does the workflow avoid storing raw provider payloads?

WF-REV-013. Does the workflow avoid becoming the product data authority?

WF-REV-014. Does the direct-gRPC rubric justify any bypass?

WF-REV-015. Does promote evidence cite `check-workflow-substrate-engine`?

## Extended Workflow Evidence Ledger

WF-EVID-001. Record workflow template id.

WF-EVID-002. Record workflow template version.

WF-EVID-003. Record owning µservice.

WF-EVID-004. Record business authority service.

WF-EVID-005. Record trigger contract path.

WF-EVID-006. Record state count.

WF-EVID-007. Record transition count.

WF-EVID-008. Record terminal state count.

WF-EVID-009. Record timer count.

WF-EVID-010. Record compensation step count.

WF-EVID-011. Record Cedar guard count.

WF-EVID-012. Record human-gate count.

WF-EVID-013. Record retry policy count.

WF-EVID-014. Record idempotency key field.

WF-EVID-015. Record outbox topic.

WF-EVID-016. Record audit event count.

WF-EVID-017. Record replay fixture path.

WF-EVID-018. Record invalid-transition fixture path.

WF-EVID-019. Record stuck-workflow runbook path.

WF-EVID-020. Record latency SLO path.

WF-EVID-021. Record direct-gRPC exception count.

WF-EVID-022. Record pack-overlay timer overrides.

WF-EVID-023. Record residency cell binding.

WF-EVID-024. Record data-class context fields.

WF-EVID-025. Record OpenAPI port path.

WF-EVID-026. Record AsyncAPI event path.

WF-EVID-027. Record Proto port path.

WF-EVID-028. Record checker crate version.

WF-EVID-029. Record VCS changeset id.

WF-EVID-030. Record promote bundle id.

## Extended Workflow Failure Modes

WF-FAIL-001. Transition performs side effect before idempotency write.

WF-FAIL-002. Timer expiry has no terminal path.

WF-FAIL-003. Compensation step is declared but not tested.

WF-FAIL-004. Human approval lacks Cedar actor check.

WF-FAIL-005. Retry loop has no max attempts.

WF-FAIL-006. Workflow state stores provider plaintext.

WF-FAIL-007. Workflow engine becomes product data authority.

WF-FAIL-008. Replay produces different final state.

WF-FAIL-009. Terminal state omits audit event.

WF-FAIL-010. Direct call bypasses workflow without rubric citation.

## Extended Promotion Review Checklist

WF-PROMOTE-001. Workflow template id is stable.

WF-PROMOTE-002. Workflow template version is explicit.

WF-PROMOTE-003. Owning µservice is cited.

WF-PROMOTE-004. Business authority service is cited.

WF-PROMOTE-005. Trigger contract path is cited.

WF-PROMOTE-006. Every state has an invariant.

WF-PROMOTE-007. Every transition has a guard.

WF-PROMOTE-008. Every terminal state emits audit.

WF-PROMOTE-009. Every timer has expiry behavior.

WF-PROMOTE-010. Every compensation path is tested.

WF-PROMOTE-011. Every Cedar guard is fixture-tested.

WF-PROMOTE-012. Every human gate has actor evidence.

WF-PROMOTE-013. Every retry has max attempts.

WF-PROMOTE-014. Idempotency key is stored first.

WF-PROMOTE-015. Outbox topic is declared.

WF-PROMOTE-016. Replay fixture passes.

WF-PROMOTE-017. Invalid-transition fixture passes.

WF-PROMOTE-018. Stuck-workflow runbook is linked.

WF-PROMOTE-019. Latency SLO is linked.

WF-PROMOTE-020. Direct-gRPC exception count is zero or justified.

WF-PROMOTE-021. Pack timer overrides are explicit.

WF-PROMOTE-022. Residency cell binding is explicit.

WF-PROMOTE-023. Data-class context fields are labeled.

WF-PROMOTE-024. OpenAPI port path is cited.

WF-PROMOTE-025. AsyncAPI event path is cited.

WF-PROMOTE-026. Proto port path is cited.

WF-PROMOTE-027. Checker crate version is recorded.

WF-PROMOTE-028. VCS changeset id is recorded.

WF-PROMOTE-029. Promote bundle id is recorded.

WF-PROMOTE-030. Workflow checker output is attached.
