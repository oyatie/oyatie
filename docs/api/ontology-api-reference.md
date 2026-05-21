---
doc_class: APIReference
microservice: ontology
version: 1.0.0
status: Accepted
date: 2026-05-20
owner: axis-ontology + council-data-model + ops-platform
openapi_version: 3.2.0
asyncapi_version: 3.1.0
proto3: true
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# ontology API Reference

Canonical REST, gRPC, and AsyncAPI reference for the `ontology`
microservice. The service owns object types, object instances, link types,
link traversals, action types, action invocations, function evaluations, agent
tool specs, cross-pillar grants, audit-chain roots, and generated OpenAPI
projection for ontology-backed tools.

Contract status legend:

- `contract-bound`: implemented in the current OpenAPI, AsyncAPI, or proto3 file.
- `reference-planned`: canonical API surface derived from the PRD, pending contract promotion.

## Quick Start

Named example: `RegisterObjectLinkAndInvokeAction`.

1. Register or load an object type with `POST /object-types` or `GET /object-types/{object_type}`.
2. Create an object instance and a link with `POST /object-types/{object_type}/instances` and `POST /link-types/{link_type}/instances`.
3. Invoke an action with `POST /action-types/{action_type}/invocations` and subscribe to `ontology.events.action-type-invoked`.

Minimum headers:

- `Authorization: Bearer <oidc-token>`
- `X-Tenant-Id: <uuid-v7>`
- `X-Context-Kind: Personal | Professional`
- `Idempotency-Key: <ulid>` on mutating requests
- `X-Request-Id: <ulid>` for audit and lineage
- `Content-Type: application/json`

Example:

```http
POST /object-types/task/instances HTTP/2
Host: ontology.oyatie.dev
Authorization: Bearer eyJ...
X-Tenant-Id: 018f7a54-3ef5-7c42-a111-a2c4ad7f88f0
X-Context-Kind: Professional
Idempotency-Key: 01HYONTCREATE00000000000
Content-Type: application/json
```

## Authentication & Authorization

Authentication patterns:

- OIDC bearer for tenant-facing ontology and agent gateway calls.
- SPIFFE SVID mTLS for internal pillar services.
- Cedar authorization over object type, action type, link type, and context kind.
- Signed tool-call receipts for agent invocations.
- Audit-chain proof binding for schema mutation and cross-pillar grant records.

Principal types:

- `OntologyDesigner`: principal allowed to define object, link, action, and function types.
- `OntologyReader`: principal allowed to inspect schema and permitted instances.
- `ObjectInstanceWriter`: principal allowed to create or mutate object instances.
- `ActionInvoker`: user or service principal allowed to invoke action types.
- `FunctionEvaluator`: service principal allowed to evaluate function types.
- `AgentToolCaller`: delegated agent principal with scoped tool-call authority.
- `CrossPillarGrantIssuer`: principal allowed to issue cross-pillar access grants.
- `OntologyAuditor`: read-only evidence and audit-chain principal.

Named Cedar policy patterns:

- `ontology::tenant_scope_match`: tenant in token, object, link, and request must match.
- `ontology::context_isolation`: Personal and Professional objects remain isolated.
- `ontology::object_type_register`: object type registration requires designer authority.
- `ontology::object_instance_write`: instance mutation requires type-specific write policy.
- `ontology::link_traversal_read`: link traversal requires read on source, link, and target type.
- `ontology::action_invoke`: action invocation requires action type and target object authority.
- `ontology::agent_tool_call`: agent tool calls must be delegated and bounded by spec.
- `ontology::cross_pillar_grant`: cross-pillar grant issuance requires explicit purpose.

Authorization failure shape:

```json
{
  "error": {
    "code": "ONTOLOGY_AUTHZ_DENIED",
    "message": "Cedar policy denied ontology action",
    "request_id": "01HYREQ...",
    "details": [{"policy": "ontology::link_traversal_read"}]
  }
}
```

## REST Endpoints

### Object Types

#### GET /object-types

- Status: `contract-bound`.
- Operation: `listObjectTypes`.
- Query schema: `tenant_id`, `namespace`, `state`, `cursor`, `limit`.
- Response schema: `ListObjectTypesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `ONTOLOGY_OBJECT_TYPE_QUERY_INVALID`.

#### POST /object-types

- Status: `contract-bound`.
- Operation: `registerObjectType`.
- Request schema: `RegisterObjectTypeRequest`.
- Required fields: `object_type`, `schema`, `display`, `policy_refs`.
- Response schema: `ObjectTypeDescriptor`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_OBJECT_TYPE_ALREADY_EXISTS`.

#### GET /object-types/{object_type}

- Status: `contract-bound`.
- Operation: `getObjectType`.
- Path schema: `object_type` as slug.
- Response schema: `ObjectTypeDescriptor`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `ONTOLOGY_OBJECT_TYPE_NOT_FOUND`.

#### PATCH /object-types/{object_type}

- Status: `reference-planned`.
- Operation: `updateObjectType`.
- Request schema: `UpdateObjectTypeRequest`.
- Required fields: `expected_version`, `schema_patch`, `migration_policy`.
- Response schema: `ObjectTypeDescriptor`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_OBJECT_TYPE_VERSION_CONFLICT`.

#### GET /object-types/{object_type}/versions

- Status: `reference-planned`.
- Operation: `listObjectTypeVersions`.
- Query schema: `cursor`, `limit`, `from_time`, `to_time`.
- Response schema: `ListObjectTypeVersionsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `ONTOLOGY_OBJECT_TYPE_VERSION_QUERY_INVALID`.

### Object Instances

#### GET /object-types/{object_type}/instances

- Status: `contract-bound`.
- Operation: `listObjectInstances`.
- Query schema: `tenant_id`, `filter`, `cursor`, `limit`, `include_tombstoned`.
- Response schema: `ListObjectInstancesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `ONTOLOGY_INSTANCE_QUERY_INVALID`.

#### POST /object-types/{object_type}/instances

- Status: `contract-bound`.
- Operation: `createObjectInstance`.
- Request schema: `CreateObjectInstanceRequest`.
- Required fields: `object_id`, `properties`, `context_kind`.
- Response schema: `ObjectInstance`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_OBJECT_INSTANCE_ALREADY_EXISTS`.

#### GET /object-types/{object_type}/instances/{object_id}

- Status: `contract-bound`.
- Operation: `getObjectInstance`.
- Path schema: `object_type`, `object_id`.
- Response schema: `ObjectInstance`.
- Status codes: `200`, `401`, `403`, `404`, `410`, `429`, `500`.
- Error shape: `ONTOLOGY_OBJECT_INSTANCE_NOT_FOUND`.

#### PATCH /object-types/{object_type}/instances/{object_id}

- Status: `contract-bound`.
- Operation: `updateObjectInstance`.
- Request schema: `UpdateObjectInstanceRequest`.
- Required fields: `expected_version`, `property_patch`.
- Response schema: `ObjectInstance`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_OBJECT_INSTANCE_VERSION_CONFLICT`.

#### DELETE /object-types/{object_type}/instances/{object_id}

- Status: `contract-bound`.
- Operation: `tombstoneObjectInstance`.
- Request schema: `TombstoneObjectInstanceRequest`.
- Required fields: `reason`, `expected_version`.
- Response schema: `ObjectTombstoneReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `ONTOLOGY_OBJECT_INSTANCE_HOLD_BLOCKED`.

### Link Types and Traversal

#### GET /link-types

- Status: `contract-bound`.
- Operation: `listLinkTypes`.
- Query schema: `tenant_id`, `namespace`, `source_type`, `target_type`, `cursor`, `limit`.
- Response schema: `ListLinkTypesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `ONTOLOGY_LINK_TYPE_QUERY_INVALID`.

#### POST /link-types

- Status: `reference-planned`.
- Operation: `registerLinkType`.
- Request schema: `RegisterLinkTypeRequest`.
- Required fields: `link_type`, `source_type`, `target_type`, `cardinality`, `policy_refs`.
- Response schema: `LinkTypeDescriptor`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_LINK_TYPE_ALREADY_EXISTS`.

#### POST /link-types/{link_type}/instances

- Status: `contract-bound`.
- Operation: `createLinkInstance`.
- Request schema: `CreateLinkInstanceRequest`.
- Required fields: `src_object_id`, `target_object_id`, `properties`.
- Response schema: `LinkInstance`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_LINK_INSTANCE_INVALID`.

#### GET /link-types/{link_type}/traversals/{src_object_id}

- Status: `contract-bound`.
- Operation: `traverseLink`.
- Path schema: `link_type`, `src_object_id`.
- Query schema: `depth`, `direction`, `cursor`, `limit`.
- Response schema: `LinkTraversalResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `ONTOLOGY_TRAVERSAL_DENIED`.

#### DELETE /link-types/{link_type}/instances/{link_id}

- Status: `reference-planned`.
- Operation: `deleteLinkInstance`.
- Request schema: `DeleteLinkInstanceRequest`.
- Required fields: `reason`, `expected_version`.
- Response schema: `LinkDeletionReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `ONTOLOGY_LINK_INSTANCE_NOT_FOUND`.

### Action Types

#### GET /action-types

- Status: `contract-bound`.
- Operation: `listActionTypes`.
- Query schema: `tenant_id`, `namespace`, `object_type`, `cursor`, `limit`.
- Response schema: `ListActionTypesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `ONTOLOGY_ACTION_TYPE_QUERY_INVALID`.

#### POST /action-types

- Status: `reference-planned`.
- Operation: `registerActionType`.
- Request schema: `RegisterActionTypeRequest`.
- Required fields: `action_type`, `input_schema`, `effect_schema`, `policy_refs`.
- Response schema: `ActionTypeDescriptor`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_ACTION_TYPE_ALREADY_EXISTS`.

#### POST /action-types/{action_type}/invocations

- Status: `contract-bound`.
- Operation: `invokeAction`.
- Request schema: `InvokeActionRequest`.
- Required fields: `target_ref`, `input`, `idempotency_key`.
- Response schema: `ActionInvocationReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_ACTION_INVOCATION_DENIED`.

#### GET /action-types/{action_type}/receipts/{receipt_id}

- Status: `reference-planned`.
- Operation: `getActionReceipt`.
- Path schema: `action_type`, `receipt_id`.
- Response schema: `ActionInvocationReceipt`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `ONTOLOGY_ACTION_RECEIPT_NOT_FOUND`.

### Function Types

#### GET /function-types

- Status: `reference-planned`.
- Operation: `listFunctionTypes`.
- Query schema: `tenant_id`, `namespace`, `object_type`, `cursor`, `limit`.
- Response schema: `ListFunctionTypesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `ONTOLOGY_FUNCTION_TYPE_QUERY_INVALID`.

#### POST /function-types

- Status: `reference-planned`.
- Operation: `registerFunctionType`.
- Request schema: `RegisterFunctionTypeRequest`.
- Required fields: `function_type`, `input_schema`, `output_schema`, `policy_refs`.
- Response schema: `FunctionTypeDescriptor`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_FUNCTION_TYPE_ALREADY_EXISTS`.

#### POST /function-types/{function_type}/evaluations

- Status: `contract-bound`.
- Operation: `evaluateFunction`.
- Request schema: `EvaluateFunctionRequest`.
- Required fields: `input`, `context_refs`, `evaluation_policy`.
- Response schema: `FunctionEvaluationResult`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_FUNCTION_EVALUATION_FAILED`.

### Agent Gateway

#### GET /agent/tool-specs

- Status: `contract-bound`.
- Operation: `listAgentToolSpecs`.
- Query schema: `tenant_id`, `object_type`, `capability`, `cursor`, `limit`.
- Response schema: `ListAgentToolSpecsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `ONTOLOGY_TOOL_SPEC_QUERY_INVALID`.

#### POST /agent/tool-call

- Status: `contract-bound`.
- Operation: `invokeAgentToolCall`.
- Request schema: `InvokeAgentToolCallRequest`.
- Required fields: `tool_name`, `arguments`, `delegation_token`.
- Response schema: `AgentToolCallReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_AGENT_TOOL_CALL_DENIED`.

#### GET /agent/tool-call/{call_id}

- Status: `reference-planned`.
- Operation: `getAgentToolCall`.
- Path schema: `call_id` as UUID-v7.
- Response schema: `AgentToolCallReceipt`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `ONTOLOGY_AGENT_TOOL_CALL_NOT_FOUND`.

### Audit and Projection

#### GET /audit-chain/{tenant_id}/{period}

- Status: `contract-bound`.
- Operation: `getAuditChainRoot`.
- Path schema: `tenant_id`, `period`.
- Response schema: `OntologyAuditRoot`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `ONTOLOGY_AUDIT_ROOT_NOT_FOUND`.

#### GET /openapi.yaml

- Status: `contract-bound`.
- Operation: `getOpenApiProjection`.
- Query schema: `tenant_id`, `namespace`, `include_agent_tools`.
- Response schema: `OpenAPI 3.2.0 document`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `ONTOLOGY_OPENAPI_PROJECTION_FAILED`.

#### POST /cross-pillar-grants

- Status: `reference-planned`.
- Operation: `issueCrossPillarGrant`.
- Request schema: `IssueCrossPillarGrantRequest`.
- Required fields: `source_pillar`, `target_pillar`, `scope`, `purpose`, `expires_at`.
- Response schema: `CrossPillarGrant`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `ONTOLOGY_CROSS_PILLAR_GRANT_DENIED`.

### Health

#### GET /health

- Status: `contract-bound`.
- Operation: `health`.
- Response schema: `HealthStatus`.
- Status codes: `200`, `500`.
- Error shape: standard health probe failure.

#### GET /ready

- Status: `contract-bound`.
- Operation: `ready`.
- Response schema: `ReadinessStatus`.
- Status codes: `200`, `503`.
- Error shape: `ONTOLOGY_SCHEMA_STORE_UNREADY`.

## gRPC Methods

### service ObjectTypeRegistryService

```proto
rpc ListObjectTypes(ListObjectTypesRequest) returns (ListObjectTypesResponse);
```

- Status: `contract-bound`.
- Semantics: lists object type descriptors visible to the tenant.
- Auth: `ontology::tenant_scope_match`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

```proto
rpc GetObjectType(GetObjectTypeRequest) returns (ObjectTypeDescriptor);
```

- Status: `contract-bound`.
- Semantics: returns one object type descriptor.
- Auth: `ontology::tenant_scope_match`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

```proto
rpc RegisterObjectType(RegisterObjectTypeRequest) returns (ObjectTypeDescriptor);
```

- Status: `contract-bound`.
- Semantics: registers a new object type and schema policy refs.
- Auth: `ontology::object_type_register`.
- Errors: `ALREADY_EXISTS`, `INVALID_ARGUMENT`, `FAILED_PRECONDITION`.

### service EntityStoreService

```proto
rpc CreateObjectInstance(CreateObjectInstanceRequest) returns (ObjectInstance);
```

- Status: `contract-bound`.
- Semantics: creates an object instance with type validation.
- Auth: `ontology::object_instance_write`.
- Errors: `ALREADY_EXISTS`, `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

```proto
rpc GetObjectInstance(GetObjectInstanceRequest) returns (ObjectInstance);
```

- Status: `contract-bound`.
- Semantics: returns one object instance.
- Auth: `ontology::tenant_scope_match`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

```proto
rpc UpdateObjectInstance(UpdateObjectInstanceRequest) returns (ObjectInstance);
```

- Status: `contract-bound`.
- Semantics: applies an optimistic concurrency patch.
- Auth: `ontology::object_instance_write`.
- Errors: `ABORTED`, `INVALID_ARGUMENT`.

```proto
rpc TombstoneObjectInstance(TombstoneObjectInstanceRequest) returns (ObjectTombstoneReceipt);
```

- Status: `contract-bound`.
- Semantics: tombstones an instance and records audit evidence.
- Auth: `ontology::object_instance_write`.
- Errors: `FAILED_PRECONDITION`, `NOT_FOUND`.

```proto
rpc ListObjectInstances(ListObjectInstancesRequest) returns (ListObjectInstancesResponse);
```

- Status: `contract-bound`.
- Semantics: lists instances by object type and filter.
- Auth: `ontology::tenant_scope_match`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

### service LinkStoreService

```proto
rpc CreateLinkInstance(CreateLinkInstanceRequest) returns (LinkInstance);
```

- Status: `contract-bound`.
- Semantics: creates a typed link between object instances.
- Auth: `ontology::object_instance_write`.
- Errors: `INVALID_ARGUMENT`, `FAILED_PRECONDITION`.

```proto
rpc TraverseLink(TraverseLinkRequest) returns (LinkTraversalResponse);
```

- Status: `contract-bound`.
- Semantics: traverses links from a source object.
- Auth: `ontology::link_traversal_read`.
- Errors: `PERMISSION_DENIED`, `OUT_OF_RANGE`.

### service ActionEngineService

```proto
rpc InvokeAction(InvokeActionRequest) returns (ActionInvocationReceipt);
```

- Status: `contract-bound`.
- Semantics: invokes a registered action type.
- Auth: `ontology::action_invoke`.
- Errors: `FAILED_PRECONDITION`, `PERMISSION_DENIED`.

### service FunctionEngineService

```proto
rpc EvaluateFunction(EvaluateFunctionRequest) returns (FunctionEvaluationResult);
```

- Status: `contract-bound`.
- Semantics: evaluates a registered function type against context refs.
- Auth: `ontology::action_invoke`.
- Errors: `INVALID_ARGUMENT`, `FAILED_PRECONDITION`.

### service AgentGatewayService

```proto
rpc ListToolSpecs(ListToolSpecsRequest) returns (ListAgentToolSpecsResponse);
```

- Status: `contract-bound`.
- Semantics: lists agent-callable tools generated from ontology descriptors.
- Auth: `ontology::agent_tool_call`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

```proto
rpc InvokeToolCall(InvokeAgentToolCallRequest) returns (AgentToolCallReceipt);
```

- Status: `contract-bound`.
- Semantics: invokes an ontology-backed tool through delegated authority.
- Auth: `ontology::agent_tool_call`.
- Errors: `FAILED_PRECONDITION`, `PERMISSION_DENIED`.

### service AuditChainService

```proto
rpc GetMerkleRoot(GetAuditChainRootRequest) returns (OntologyAuditRoot);
```

- Status: `contract-bound`.
- Semantics: returns ontology audit root for tenant and period.
- Auth: `ontology::tenant_scope_match`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

## AsyncAPI Channels

### ontology.events.object-type-registered

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `ObjectTypeRegistered`.
- Delivery semantics: at-least-once, compacted by `object_type`.
- Consumers: governance, workflow-studio, agent gateway.

### ontology.events.object-instance-mutated

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `ObjectInstanceMutated`.
- Delivery semantics: ordered per `object_id`.
- Consumers: audit-chain, search, workflow-runtime.

### ontology.events.link-type-registered

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `LinkTypeRegistered`.
- Delivery semantics: at-least-once, compacted by `link_type`.
- Consumers: governance, agent gateway, workflow-studio.

### ontology.events.action-type-registered

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `ActionTypeRegistered`.
- Delivery semantics: at-least-once, compacted by `action_type`.
- Consumers: agent gateway, governance.

### ontology.events.action-type-invoked

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `ActionTypeInvoked`.
- Delivery semantics: at-least-once with `receipt_id`.
- Consumers: audit-chain, governance, workflow-runtime.

### ontology.events.audit-chain-sealed

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `OntologyAuditChainSealed`.
- Delivery semantics: ordered per tenant and period.
- Consumers: governance, audit-chain, compliance views.

### ontology.events.cross-pillar-grant-requested

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `CrossPillarGrantRequested`.
- Delivery semantics: at-least-once.
- Consumers: governance, privacy, approval workflows.

### ontology.events.cross-pillar-grant-issued

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `CrossPillarGrantIssued`.
- Delivery semantics: at-least-once with grant id.
- Consumers: access control, governance, audit-chain.

### ontology.events.schema-migration-completed

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `SchemaMigrationCompleted`.
- Delivery semantics: at-least-once, ordered per `object_type`.
- Consumers: search, workflow-studio, governance.

### ontology.events.dsr-erasure-executed

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `DsrErasureExecuted`.
- Delivery semantics: at-least-once with redaction receipt.
- Consumers: privacy, audit-chain, governance.

### governance.policy-pack-updated

- Direction: subscribe.
- Status: `reference-planned`.
- Payload schema: `PolicyPackUpdated`.
- Delivery semantics: compacted by `pack_id`.
- Handler: refresh Cedar policy references and grants.

### audit-chain.seal-minted

- Direction: subscribe.
- Status: `reference-planned`.
- Payload schema: `SealMinted`.
- Delivery semantics: ordered per period.
- Handler: attach external chain proof to ontology root.

## Webhooks Inbound

### webhook.schema.registry-updated

- Source: schema registry.
- Event: `schema.registry_updated`.
- Payload schema: `SchemaRegistryUpdatedWebhook`.
- Semantics: refreshes object, link, action, and function schema projections.

### webhook.governance.policy-pack-updated

- Source: governance.
- Event: `governance.policy_pack.updated`.
- Payload schema: `PolicyPackUpdatedWebhook`.
- Semantics: refreshes policy refs and action admission checks.

### webhook.audit-chain.seal-minted

- Source: audit-chain.
- Event: `audit_chain.seal_minted`.
- Payload schema: `SealMintedWebhook`.
- Semantics: binds ontology audit root to external chain proof.

### webhook.privacy.dsr-erasure-approved

- Source: privacy service.
- Event: `privacy.dsr_erasure.approved`.
- Payload schema: `DsrErasureApprovedWebhook`.
- Semantics: tombstones or redacts permitted instances.

### webhook.agent.delegation-revoked

- Source: agent gateway.
- Event: `agent.delegation.revoked`.
- Payload schema: `AgentDelegationRevokedWebhook`.
- Semantics: invalidates delegated tool-call tokens.

### webhook.workflow.action-invocation-requested

- Source: workflow-runtime.
- Event: `workflow.action_invocation.requested`.
- Payload schema: `ActionInvocationRequestedWebhook`.
- Semantics: invokes ontology action type on behalf of workflow runtime.

### webhook.drive.object-indexed

- Source: drive.
- Event: `drive.object_indexed`.
- Payload schema: `DriveObjectIndexedWebhook`.
- Semantics: updates file-backed object instance descriptors.

### webhook.messenger.thread-linked

- Source: messenger.
- Event: `messenger.thread_linked`.
- Payload schema: `MessengerThreadLinkedWebhook`.
- Semantics: creates link instances between conversations and ontology objects.

## SDK Quick Reference

### Rust

```rust
let object_type = ontology::register_object_type(client, descriptor).await?;
let object = ontology::create_object_instance(client, object_type.name, properties).await?;
let link = ontology::create_link_instance(client, link_type, source, target).await?;
let receipt = ontology::invoke_action(client, action_type, input).await?;
let tools = ontology::list_agent_tool_specs(client, query).await?;
```

Named functions:

- `list_object_types`
- `register_object_type`
- `get_object_type`
- `update_object_type`
- `list_object_instances`
- `create_object_instance`
- `update_object_instance`
- `tombstone_object_instance`
- `create_link_instance`
- `traverse_link`
- `invoke_action`
- `evaluate_function`
- `list_agent_tool_specs`
- `invoke_agent_tool_call`

### TypeScript

```ts
const ontology = new OntologyClient({ tenantId, token });
const descriptor = await ontology.registerObjectType({ objectType, schema });
const object = await ontology.createObjectInstance(objectType, { properties });
await ontology.createLinkInstance(linkType, { srcObjectId, targetObjectId });
const receipt = await ontology.invokeAction(actionType, { targetRef, input });
const specs = await ontology.listAgentToolSpecs({ objectType });
```

Named functions:

- `listObjectTypes`
- `registerObjectType`
- `getObjectType`
- `updateObjectType`
- `listObjectInstances`
- `createObjectInstance`
- `updateObjectInstance`
- `tombstoneObjectInstance`
- `createLinkInstance`
- `traverseLink`
- `invokeAction`
- `evaluateFunction`
- `listAgentToolSpecs`
- `invokeAgentToolCall`

### Python

```python
ontology = OntologyClient(tenant_id=tenant_id, token=token)
descriptor = ontology.register_object_type(object_type=object_type, schema=schema)
obj = ontology.create_object_instance(object_type, properties=properties)
ontology.create_link_instance(link_type, src_object_id=src, target_object_id=target)
receipt = ontology.invoke_action(action_type, target_ref=target, input=input_data)
tools = ontology.list_agent_tool_specs(object_type=object_type)
```

Named functions:

- `list_object_types`
- `register_object_type`
- `get_object_type`
- `update_object_type`
- `list_object_instances`
- `create_object_instance`
- `update_object_instance`
- `tombstone_object_instance`
- `create_link_instance`
- `traverse_link`
- `invoke_action`
- `evaluate_function`
- `list_agent_tool_specs`
- `invoke_agent_tool_call`

## Error Catalogue

### ONTOLOGY_AUTHZ_DENIED

- Meaning: Cedar denied ontology operation.
- Retry policy: do not retry without changing scope or principal.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### ONTOLOGY_OBJECT_TYPE_ALREADY_EXISTS

- Meaning: object type slug already exists.
- Retry policy: fetch existing descriptor or use a new type name.
- HTTP mapping: `409`.
- gRPC mapping: `ALREADY_EXISTS`.

### ONTOLOGY_OBJECT_TYPE_VERSION_CONFLICT

- Meaning: expected descriptor version does not match current version.
- Retry policy: reload descriptor and retry after merge.
- HTTP mapping: `409`.
- gRPC mapping: `ABORTED`.

### ONTOLOGY_OBJECT_INSTANCE_ALREADY_EXISTS

- Meaning: object id already exists for the object type.
- Retry policy: safe to fetch existing instance if idempotency key matches.
- HTTP mapping: `409`.
- gRPC mapping: `ALREADY_EXISTS`.

### ONTOLOGY_OBJECT_INSTANCE_VERSION_CONFLICT

- Meaning: optimistic concurrency version mismatch.
- Retry policy: reload object instance and reapply patch.
- HTTP mapping: `409`.
- gRPC mapping: `ABORTED`.

### ONTOLOGY_OBJECT_INSTANCE_HOLD_BLOCKED

- Meaning: legal hold or retention policy blocks tombstone.
- Retry policy: do not retry until hold state changes.
- HTTP mapping: `409`.
- gRPC mapping: `FAILED_PRECONDITION`.

### ONTOLOGY_LINK_INSTANCE_INVALID

- Meaning: link violates type, cardinality, or target policy.
- Retry policy: fix link request and retry.
- HTTP mapping: `422`.
- gRPC mapping: `INVALID_ARGUMENT`.

### ONTOLOGY_TRAVERSAL_DENIED

- Meaning: caller lacks read permission for traversal path.
- Retry policy: do not retry without grant or narrower traversal.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### ONTOLOGY_ACTION_INVOCATION_DENIED

- Meaning: action type policy denied invocation.
- Retry policy: do not retry without changing target, input, or authority.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### ONTOLOGY_FUNCTION_EVALUATION_FAILED

- Meaning: function evaluation failed validation or execution policy.
- Retry policy: retry only after correcting input or dependency state.
- HTTP mapping: `422`.
- gRPC mapping: `FAILED_PRECONDITION`.

### ONTOLOGY_AGENT_TOOL_CALL_DENIED

- Meaning: delegated agent tool call exceeded token or spec bounds.
- Retry policy: do not retry without a new delegation or narrower arguments.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### ONTOLOGY_RATE_LIMITED

- Meaning: capability-tier quota was exceeded.
- Retry policy: honor `Retry-After`; bulk sync should checkpoint and resume.
- HTTP mapping: `429`.
- gRPC mapping: `RESOURCE_EXHAUSTED`.

## Pagination

Cursor pattern name: `ontology_type_scoped_cursor`.

Cursor fields:

- `tenant_id`
- `context_kind`
- `resource_kind`
- `type_name`
- `sort_key`
- `last_seen_id`
- `issued_at`
- `signature`

Rules:

- Cursor values are opaque and signed.
- Object instances sort by `updated_at` and UUID-v7 tiebreaker.
- Link traversals sort by traversal order and link id.
- Agent tool specs may be compacted by generated spec version.
- Cursor TTL is 15 minutes for instance views and 60 minutes for schema views.
- Invalid cursors return `ONTOLOGY_CURSOR_INVALID`.

Max page-size limits:

- Object types: `200`.
- Object type versions: `200`.
- Object instances: `500`.
- Link types: `200`.
- Link traversals: `500`.
- Action types: `200`.
- Function types: `200`.
- Agent tool specs: `200`.
- Default page size: `100`.

## Rate Limits per Tier

Per ADR-0316, ontology uses capability-tier throttles rather than
product-fragmented limits.

| Tier | REST requests per second | gRPC requests per second | Async publishes per second | Burst |
| --- | ---: | ---: | ---: | ---: |

Special limits:


## OpenAPI 3.2.0 Schema

Actual contracts file:

- [ontology.yaml](../../microservices/ontology/contracts/openapi/ontology.yaml)

Design references:

- [ontology PRD](../../microservices/ontology/PRD.md)
- [API design standard](../standards/api-design.md)
- [Throttling tiers](../standards/throttling-tiers.md)

## AsyncAPI 3.1.0 Schema

Actual contracts file:

- [ontology-events.yaml](../../microservices/ontology/contracts/asyncapi/ontology-events.yaml)

Delivery notes:

- Object instance mutations are ordered per `object_id`.
- Type registration events may be compacted by type name.
- Action invocation events include receipt id for deduplication.
- Cross-pillar grants must be audit-sealed before activation.

## proto3 Schema

Actual contracts file:

- [ontology.proto](../../microservices/ontology/contracts/proto/ontology.proto)

Proto package expectations:

- Use proto3 syntax.
- Keep generated tool-call specs stable across SDKs.
- Map optimistic concurrency conflicts to `ABORTED`.
- Map legal hold blocks to `FAILED_PRECONDITION`.

## Cross-References

- [ontology PRD](../../microservices/ontology/PRD.md)
- [ADR-0316 capability tier over product fragmentation](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md)
- [API design standard](../standards/api-design.md)
- [Throttling tiers](../standards/throttling-tiers.md)
- [Governance API reference](governance-api-reference.md)
- [Audit-chain API reference](audit-chain-api-reference.md)
- [Workflow Studio API reference](workflow-studio-api-reference.md)
- [Drive API reference](drive-api-reference.md)
