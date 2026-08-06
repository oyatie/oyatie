---
doc_class: Standard
title: AsyncAPI 3.1 Authoring Standard
status: Accepted
date: 2026-05-20
owner: axis-eventing + council-architecture
related_oyatie_adrs:
  - ADR-0005
  - ADR-0131
  - ADR-0145
  - ADR-0154
  - ADR-0263
enforced_by:
  - oya-governance-asyncapi-3-1-authoring
  - oya-governance-event-schema-versioning
  - oya-governance-audit-emission
canonical_paths:
  - microservices/*/contracts/*.asyncapi.yaml
  - docs/standards/event-schema-versioning-canonical.md
  - contracts/
external_reference:
  - https://www.asyncapi.com/docs/reference/specification/v3.1.0
---

# AsyncAPI 3.1 Authoring Standard

AsyncAPI 3.1.0 is the canonical contract format for asynchronous APIs, events,
commands, streams, and service notifications in Oyatie. This standard defines the
required authoring profile for `*.asyncapi.yaml` files so generated SDKs,
workflow templates, event processors, audit gates, and governance checkers can
consume contracts without per-service interpretation.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to every AsyncAPI file under `microservices/*/contracts/`.

It applies to event publishing contracts.

It applies to event consuming contracts.

It applies to workflow event contracts.

It applies to audit event contracts when represented through AsyncAPI.

It applies to NATS, Kafka-compatible, WebSocket, and webhook event surfaces.

It applies to generated event clients.

It applies to event schema versioning.

It applies to message examples used in tests.

It does not replace CloudEvents or audit-chain event schemas.

It does not define REST endpoints.

It does not define proto3 internal service contracts.

## Normative Requirements

A-001. Every async contract MUST declare AsyncAPI version 3.1.0.

A-002. Every async contract MUST declare `info.title`.

A-003. Every async contract MUST declare `info.version`.

A-004. Every async contract MUST declare owning microservice.

A-005. Every async contract MUST declare default content type.

A-006. Every async contract MUST declare servers or explicit serverless posture.

A-007. Every channel MUST have a stable address.

A-008. Every channel address MUST include version when it is public.

A-009. Every channel MUST declare messages.

A-010. Every message MUST declare a stable message id.

A-011. Every message MUST declare payload schema.

A-012. Every message MUST declare headers schema when headers exist.

A-013. Every message MUST declare correlation id.

A-014. Every message MUST declare idempotency key when mutation can result.

A-015. Every message MUST declare tenant id or tenant scope posture.

A-016. Every message MUST declare principal or system principal posture.

A-017. Every message MUST declare data class.

A-018. Every message MUST declare audit event class when state mutation occurs.

A-019. Every message MUST declare ordering key when ordering matters.

A-020. Every message MUST declare replay semantics.

A-021. Every message MUST declare retention expectations.

A-022. Every message MUST declare dead-letter behavior.

A-023. Every message MUST declare retry classification.

A-024. Every operation MUST declare send or receive action.

A-025. Every operation MUST bind to one or more channels.

A-026. Every operation MUST declare security requirements.

A-027. Every operation MUST declare tags.

A-028. Every operation SHOULD declare external docs pointing to runbooks.

A-029. Every schema MUST be versioned.

A-030. Every schema MUST avoid untyped free-form payloads.

A-031. Every schema MUST avoid unbounded maps unless justified.

A-032. Every schema MUST reserve extension fields explicitly.

A-033. Every schema MUST declare nullable fields deliberately.

A-034. Every schema MUST declare enum evolution policy.

A-035. Every schema MUST declare backward compatibility.

A-036. Every breaking message change MUST increment major version.

A-037. Every non-breaking message change MUST preserve old consumers.

A-038. Every deprecated channel MUST declare sunset.

A-039. Every deleted channel MUST have migration evidence.

A-040. Every example MUST be valid against schema.

A-041. Every example MUST include tenant, request, and correlation fields when applicable.

A-042. Every event that crosses tenant boundary MUST be forbidden unless a pack explicitly permits it.

A-043. Every event that carries regulated data MUST declare pack overlays.

A-044. Every event that drives workflow MUST reference workflow template id.

A-045. Every event that updates ontology MUST reference ontology projection id.

A-046. Every event that triggers billing MUST reference cost dimensions.

A-047. Every event producer MUST be named.

A-048. Every event consumer MUST be named or declared open-consumer.

A-049. Every open-consumer event MUST be public and versioned.

A-050. Every internal event SHOULD still be versioned.

A-051. AsyncAPI files MUST NOT use ambiguous `events.yaml` names.

A-052. AsyncAPI files MUST NOT mix unrelated bounded contexts.

A-053. AsyncAPI files MUST NOT omit security posture.

A-054. AsyncAPI files MUST NOT omit dead-letter posture.

A-055. AsyncAPI files MUST NOT omit replay posture.

A-056. AsyncAPI files MUST NOT duplicate OpenAPI request schemas by copy-paste when a shared schema exists.

A-057. AsyncAPI files MUST NOT use examples containing real tenant data.

A-058. AsyncAPI files MUST NOT use wildcard channels for tenant data.

A-059. AsyncAPI files MUST NOT use broker-specific semantics without binding declaration.

A-060. AsyncAPI files MUST NOT publish state mutation events without audit event mapping.

## Worked Examples

### Example 1: Workflow event channel

```yaml
asyncapi: 3.1.0
info:
  title: Workflow Engine Events
  version: 1.0.0
defaultContentType: application/json
channels:
  workflow.step.completed.v1:
    address: workflow.step.completed.v1
    messages:
      StepCompleted:
        $ref: '#/components/messages/StepCompleted'
operations:
  publishStepCompleted:
    action: send
    channel:
      $ref: '#/channels/workflow.step.completed.v1'
```

This passes because channel, message, and operation are explicit.

### Example 2: Message payload

```yaml
components:
  messages:
    StepCompleted:
      name: StepCompleted
      correlationId:
        location: $message.header#/correlation_id
      payload:
        type: object
        required: [tenant_id, workflow_id, step_id, audit_event_class]
```

This passes because correlation and audit shape are present.

### Example 3: Invalid wildcard tenant channel

```yaml
channels:
  tenant.*.messages:
    address: tenant.*.messages
```

This fails because tenant wildcard channels hide isolation.

### Example 4: Dead-letter posture

```yaml
x-oyatie-delivery:
  retry_policy: exponential-jitter
  max_attempts: 5
  dead_letter_channel: workflow.step.completed.deadletter.v1
  replay: supported
```

This passes because failed delivery has declared behavior.

### Example 5: Ontology update event

```yaml
x-oyatie-ontology:
  projection_id: ontology.object.ApprovalRoute.v1
  refresh_mode: event-driven
```

This passes because ontology dependency is explicit.

## Verification

Primary command:

```bash
oya gate validate asyncapi-3-1-authoring --scope microservices
```

The checker MUST parse all `*.asyncapi.yaml`.

The checker MUST validate AsyncAPI version.

The checker MUST validate required info fields.

The checker MUST validate channels.

The checker MUST validate operations.

The checker MUST validate messages.

The checker MUST validate schemas.

The checker MUST validate examples.

The checker MUST validate correlation ids.

The checker MUST validate tenant fields.

The checker MUST validate data class fields.

The checker MUST validate audit event mappings.

The checker MUST validate dead-letter declarations.

The checker MUST validate replay declarations.

The checker MUST validate security requirements.

The checker MUST validate workflow references.

The checker MUST validate ontology references.

The checker MUST validate channel versioning.

The checker SHOULD generate fixture events.

The checker SHOULD generate consumer compatibility reports.

## Common Anti-Patterns

Using `events.yaml` is an anti-pattern.

Using unversioned channels is an anti-pattern.

Using wildcard tenant channels is an anti-pattern.

Publishing mutation events without audit class is an anti-pattern.

Omitting correlation id is an anti-pattern.

Omitting dead-letter behavior is an anti-pattern.

Omitting replay behavior is an anti-pattern.

Using real tenant examples is an anti-pattern.

Copy-pasting OpenAPI schemas without ownership is an anti-pattern.

Using broker-specific options without binding is an anti-pattern.

Treating AsyncAPI as generated after implementation is an anti-pattern.

Treating messages as logs is an anti-pattern.

Treating consumers as unknown for internal events is an anti-pattern.

Treating schema evolution as best effort is an anti-pattern.

Treating ordering keys as obvious is an anti-pattern.

## Cross-References

External authority: `https://www.asyncapi.com/docs/reference/specification/v3.1.0`.

`docs/standards/event-schema-versioning-canonical.md` binds event versioning.

`docs/standards/workflow-substrate-engine.md` binds workflow events.

`docs/standards/ontology-projection-substrate.md` binds projection update events.

`docs/standards/openapi-3-2-authoring.md` binds REST contracts.

`docs/standards/proto3-authoring.md` binds gRPC contracts.

`docs/decisions/ADR-0709-general-live-apex.md` binds outbox eventing.

`docs/decisions/ADR-0706-observability-live-apex.md` binds audit and telemetry events.

## Substance Bar Compliance Checklist

ASYNC-SB-001. Verify AsyncAPI version 3.1.0.

ASYNC-SB-002. Verify info title.

ASYNC-SB-003. Verify info version.

ASYNC-SB-004. Verify owning microservice extension.

ASYNC-SB-005. Verify default content type.

ASYNC-SB-006. Verify servers or serverless posture.

ASYNC-SB-007. Verify channel address.

ASYNC-SB-008. Verify channel version.

ASYNC-SB-009. Verify channel messages.

ASYNC-SB-010. Verify message id.

ASYNC-SB-011. Verify payload schema.

ASYNC-SB-012. Verify headers schema.

ASYNC-SB-013. Verify correlation id.

ASYNC-SB-014. Verify idempotency key.

ASYNC-SB-015. Verify tenant field.

ASYNC-SB-016. Verify principal posture.

ASYNC-SB-017. Verify data class.

ASYNC-SB-018. Verify audit event class.

ASYNC-SB-019. Verify ordering key.

ASYNC-SB-020. Verify replay semantics.

ASYNC-SB-021. Verify retention expectation.

ASYNC-SB-022. Verify dead-letter behavior.

ASYNC-SB-023. Verify retry classification.

ASYNC-SB-024. Verify operation action.

ASYNC-SB-025. Verify operation channel binding.

ASYNC-SB-026. Verify security requirements.

ASYNC-SB-027. Verify tags.

ASYNC-SB-028. Verify external docs.

ASYNC-SB-029. Verify schema version.

ASYNC-SB-030. Verify enum evolution.

ASYNC-SB-031. Verify backward compatibility.

ASYNC-SB-032. Verify deprecation sunset.

ASYNC-SB-033. Verify example validity.

ASYNC-SB-034. Verify tenant-safe examples.

ASYNC-SB-035. Verify workflow reference.

ASYNC-SB-036. Verify ontology reference.

ASYNC-SB-037. Verify cost dimensions.

ASYNC-SB-038. Verify producer name.

ASYNC-SB-039. Verify consumer name.

ASYNC-SB-040. Verify open-consumer posture.

ASYNC-SB-041. Check `workflow.step.completed.v1`.

ASYNC-SB-042. Check `workflow.step.completed.deadletter.v1`.

ASYNC-SB-043. Check `ontology.projection.updated.v1`.

ASYNC-SB-044. Check `capability.tier.granted.v1`.

ASYNC-SB-045. Check `messenger.delivery.receipt.v1`.

ASYNC-SB-046. Check `mail.message.accepted.v1`.

ASYNC-SB-047. Check `tenancy.tenant.created.v1`.

ASYNC-SB-048. Check `policy.decision.emitted.v1`.

ASYNC-SB-049. Check `audit.event.sealed.v1`.

ASYNC-SB-050. Check `observability.slo.burn.v1`.

ASYNC-SB-051. Reject `events.yaml`.

ASYNC-SB-052. Reject unversioned channel.

ASYNC-SB-053. Reject wildcard tenant channel.

ASYNC-SB-054. Reject missing audit mapping.

ASYNC-SB-055. Reject missing correlation id.

ASYNC-SB-056. Reject missing dead-letter behavior.

ASYNC-SB-057. Reject missing replay behavior.

ASYNC-SB-058. Reject real tenant examples.

ASYNC-SB-059. Reject broker option without binding.

ASYNC-SB-060. Reject schema copy-paste drift.

ASYNC-SB-061. Emit AsyncAPI file count.

ASYNC-SB-062. Emit channel count.

ASYNC-SB-063. Emit message count.

ASYNC-SB-064. Emit operation count.

ASYNC-SB-065. Emit schema count.

ASYNC-SB-066. Emit example count.

ASYNC-SB-067. Emit dead-letter count.

ASYNC-SB-068. Emit replay count.

ASYNC-SB-069. Emit workflow binding count.

ASYNC-SB-070. Emit ontology binding count.

ASYNC-SB-071. Preserve event schema versioning.

ASYNC-SB-072. Preserve outbox discipline.

ASYNC-SB-073. Preserve audit event mapping.

ASYNC-SB-074. Preserve tenant isolation.

ASYNC-SB-075. Preserve data-class labeling.

ASYNC-SB-076. Preserve replay safety.

ASYNC-SB-077. Preserve consumer compatibility.

ASYNC-SB-078. Preserve broker portability.

ASYNC-SB-079. Preserve workflow integration.

ASYNC-SB-080. Preserve ontology projection integration.

## Extended Worked Example: Workflow Event Contract

```yaml
asyncapi: 3.1.0
info:
  title: Workflow Engine Events
  version: 1.0.0
  x-oyatie-standard: docs/standards/asyncapi-3-1-authoring.md
  x-oyatie-adrs:
    - ADR-0145
    - ADR-0258
    - ADR-0307
defaultContentType: application/json
channels:
  workflow.execution.requested.v1:
    address: workflow.execution.requested.v1
    messages:
      WorkflowExecutionRequestedV1:
        $ref: '#/components/messages/WorkflowExecutionRequestedV1'
operations:
  publishWorkflowExecutionRequestedV1:
    action: send
    channel:
      $ref: '#/channels/workflow.execution.requested.v1'
    messages:
      - $ref: '#/channels/workflow.execution.requested.v1/messages/WorkflowExecutionRequestedV1'
components:
  messages:
    WorkflowExecutionRequestedV1:
      name: WorkflowExecutionRequestedV1
      title: Workflow execution requested
      contentType: application/json
      traits:
        - $ref: '#/components/messageTraits/OyatieEventEnvelopeV1'
      payload:
        $ref: '#/components/schemas/WorkflowExecutionRequestedPayloadV1'
  messageTraits:
    OyatieEventEnvelopeV1:
      headers:
        type: object
        required:
          - event_id
          - tenant_id
          - trace_id
          - data_class
          - schema_version
        properties:
          event_id:
            type: string
          tenant_id:
            type: string
          trace_id:
            type: string
          data_class:
            type: string
          schema_version:
            const: workflow.execution.requested.v1
  schemas:
    WorkflowExecutionRequestedPayloadV1:
      type: object
      required:
        - workflow_template_id
        - idempotency_key
      properties:
        workflow_template_id:
          type: string
        idempotency_key:
          type: string
```

## Extended AsyncAPI Matrix

| ID | Concern | Requirement | Example | Checker |
|---|---|---|---|---|
| ASYNC-MAT-001 | Version | `asyncapi: 3.1.0` | root field | `oya-check-asyncapi-version` |
| ASYNC-MAT-002 | Info | standard link | `x-oyatie-standard` | `oya-check-contract-links` |
| ASYNC-MAT-003 | Info | ADR links | `x-oyatie-adrs` | `oya-check-adr-links` |
| ASYNC-MAT-004 | Channel | stable address | `workflow.execution.requested.v1` | `oya-check-channel-names` |
| ASYNC-MAT-005 | Operation | action declared | `send` | `oya-check-asyncapi-operation` |
| ASYNC-MAT-006 | Message | name versioned | `WorkflowExecutionRequestedV1` | `oya-check-message-names` |
| ASYNC-MAT-007 | Trait | envelope reused | `OyatieEventEnvelopeV1` | `oya-check-event-envelope` |
| ASYNC-MAT-008 | Header | event id | `event_id` | `oya-check-event-id` |
| ASYNC-MAT-009 | Header | tenant id | `tenant_id` | `oya-check-tenant-boundary` |
| ASYNC-MAT-010 | Header | trace id | `trace_id` | `oya-check-trace-context` |
| ASYNC-MAT-011 | Header | data class | `data_class` | `oya-check-data-class` |
| ASYNC-MAT-012 | Header | schema version | `schema_version` | `oya-check-schema-version` |
| ASYNC-MAT-013 | Payload | required idempotency key | `idempotency_key` | `oya-check-idempotency` |
| ASYNC-MAT-014 | Payload | no raw provider blob | payload schema | `oya-check-provider-blob-boundary` |
| ASYNC-MAT-015 | Compatibility | additive changes only in v1 | schema diff | `oya-check-event-compatibility` |
| ASYNC-MAT-016 | Consumer | consumer fixture exists | fixtures | `oya-check-consumer-fixtures` |
| ASYNC-MAT-017 | Replay | event is replay-safe | event metadata | `oya-check-replay-safety` |
| ASYNC-MAT-018 | Projection | projection links named | projection manifest | `oya-check-projection-linkage` |
| ASYNC-MAT-019 | Broker | no broker-specific extension as authority | extensions | `oya-check-broker-portability` |
| ASYNC-MAT-020 | Promote | evidence includes checker output | VCS bundle | `oya-vcs-admission` |

## Extended AsyncAPI Evidence Ledger

ASYNC-EVID-001. Record AsyncAPI file path.

ASYNC-EVID-002. Record AsyncAPI version.

ASYNC-EVID-003. Record contract title.

ASYNC-EVID-004. Record contract semantic version.

ASYNC-EVID-005. Record related ADR ids.

ASYNC-EVID-006. Record owning µservice.

ASYNC-EVID-007. Record channel count.

ASYNC-EVID-008. Record operation count.

ASYNC-EVID-009. Record message count.

ASYNC-EVID-010. Record schema count.

ASYNC-EVID-011. Record trait count.

ASYNC-EVID-012. Record channel addresses.

ASYNC-EVID-013. Record message names.

ASYNC-EVID-014. Record message content types.

ASYNC-EVID-015. Record event envelope trait.

ASYNC-EVID-016. Record required header list.

ASYNC-EVID-017. Record data-class header validation.

ASYNC-EVID-018. Record tenant-id header validation.

ASYNC-EVID-019. Record trace-id header validation.

ASYNC-EVID-020. Record schema-version header validation.

ASYNC-EVID-021. Record idempotency-key validation.

ASYNC-EVID-022. Record replay-safety validation.

ASYNC-EVID-023. Record consumer fixture count.

ASYNC-EVID-024. Record producer fixture count.

ASYNC-EVID-025. Record schema diff result.

ASYNC-EVID-026. Record breaking-change count.

ASYNC-EVID-027. Record broker-portability result.

ASYNC-EVID-028. Record projection-linkage result.

ASYNC-EVID-029. Record workflow-linkage result.

ASYNC-EVID-030. Record audit-event linkage result.

ASYNC-EVID-031. Record generated SDK event count.

ASYNC-EVID-032. Record documentation link check.

ASYNC-EVID-033. Record checker crate version.

ASYNC-EVID-034. Record VCS changeset id.

ASYNC-EVID-035. Record promote bundle id.

## Extended AsyncAPI Anti-Patterns

ASYNC-APX-001. Event name lacks tense.

ASYNC-APX-002. Channel address is environment-specific.

ASYNC-APX-003. Payload omits tenant id from envelope.

ASYNC-APX-004. Payload includes raw provider response blob.

ASYNC-APX-005. Consumer compatibility is assumed without fixture.

ASYNC-APX-006. Broker-specific extension becomes semantic authority.

ASYNC-APX-007. Schema version does not match channel version.

ASYNC-APX-008. Replay-unsafe event lacks explicit direct-call rubric.

ASYNC-APX-009. Projection consumes event without source mapping.

ASYNC-APX-010. Promote evidence omits AsyncAPI diff output.

## Extended Promotion Review Checklist

ASYNC-PROMOTE-001. AsyncAPI file path is stable.

ASYNC-PROMOTE-002. AsyncAPI version is 3.1.0.

ASYNC-PROMOTE-003. Contract title is recorded.

ASYNC-PROMOTE-004. Contract semantic version is recorded.

ASYNC-PROMOTE-005. Owning µservice is recorded.

ASYNC-PROMOTE-006. Related ADR ids are recorded.

ASYNC-PROMOTE-007. Channel count is recorded.

ASYNC-PROMOTE-008. Operation count is recorded.

ASYNC-PROMOTE-009. Message count is recorded.

ASYNC-PROMOTE-010. Schema count is recorded.

ASYNC-PROMOTE-011. Trait count is recorded.

ASYNC-PROMOTE-012. Channel addresses are versioned.

ASYNC-PROMOTE-013. Message names are versioned.

ASYNC-PROMOTE-014. Message content types are declared.

ASYNC-PROMOTE-015. Event envelope trait is reused.

ASYNC-PROMOTE-016. Required headers are present.

ASYNC-PROMOTE-017. Data-class header validation passes.

ASYNC-PROMOTE-018. Tenant-id header validation passes.

ASYNC-PROMOTE-019. Trace-id header validation passes.

ASYNC-PROMOTE-020. Schema-version header validation passes.

ASYNC-PROMOTE-021. Idempotency-key validation passes.

ASYNC-PROMOTE-022. Replay-safety validation passes.

ASYNC-PROMOTE-023. Consumer fixtures exist.

ASYNC-PROMOTE-024. Producer fixtures exist.

ASYNC-PROMOTE-025. Schema diff result is attached.

ASYNC-PROMOTE-026. Breaking-change count is zero or major bump exists.

ASYNC-PROMOTE-027. Broker-portability check passes.

ASYNC-PROMOTE-028. Projection-linkage check passes.

ASYNC-PROMOTE-029. Workflow-linkage check passes.

ASYNC-PROMOTE-030. Audit-event linkage check passes.

ASYNC-PROMOTE-031. Generated SDK events compile.

ASYNC-PROMOTE-032. Documentation links resolve.

ASYNC-PROMOTE-033. Checker crate version is recorded.

ASYNC-PROMOTE-034. VCS changeset id is recorded.

ASYNC-PROMOTE-035. Promote bundle id is recorded.

ASYNC-PROMOTE-036. Channel names match naming BNF.

ASYNC-PROMOTE-037. Event names include tense.

ASYNC-PROMOTE-038. Payloads omit provider raw blobs.

ASYNC-PROMOTE-039. Projection source mapping is attached.

ASYNC-PROMOTE-040. Promotion evidence includes AsyncAPI checker output.

## Extended AsyncAPI Residual-Risk Register

ASYNC-RISK-001. Residual risk: consumer treats event order as global; mitigation is per-aggregate ordering note.

ASYNC-RISK-002. Residual risk: producer emits before outbox commit; mitigation is outbox-pattern checker.

ASYNC-RISK-003. Residual risk: consumer fixture omits old schema; mitigation is N-1 compatibility fixture.

ASYNC-RISK-004. Residual risk: broker retry duplicates messages; mitigation is idempotency-key requirement.

ASYNC-RISK-005. Residual risk: projection consumes event as authority; mitigation is ontology projection checker.

ASYNC-RISK-006. Residual risk: event data class is too broad; mitigation is data-class review.

ASYNC-RISK-007. Residual risk: dead-letter path hides failures; mitigation is dead-letter SLO and runbook.

ASYNC-RISK-008. Residual risk: generated SDK drops header fields; mitigation is SDK fixture compilation.

ASYNC-RISK-009. Residual risk: event version changes without channel bump; mitigation is schema-version checker.

ASYNC-RISK-010. Residual risk: audit event linkage is incomplete; mitigation is audit-emission checker.
