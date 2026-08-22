---
doc_class: Standard
title: Proto3 Authoring Standard
status: Accepted
date: 2026-05-20
owner: council-architecture + axis-developer-sdk
related_oyatie_adrs:
  - ADR-0131
  - ADR-0145
  - ADR-0177
  - ADR-0258
  - ADR-0316
enforced_by:
  - governance-proto3-authoring
  - governance-grpc-contract-compatibility
  - governance-sdk-generation
canonical_paths:
  - microservices/*/contracts/*.proto
  - docs/standards/api-surface-separation.md
  - docs/standards/layer-enum-adr-0105.md
external_reference:
  - https://protobuf.dev/programming-guides/proto3/
---

# Proto3 Authoring Standard

Proto3 is the canonical schema language for internal gRPC contracts, worker
contracts, and language-neutral generated client surfaces where OpenAPI is not
the right shape. This standard defines the Oyatie profile for `.proto` files:
package naming, service shape, field numbering, reserved fields, compatibility,
tenant context, data classes, Cedar linkage, audit events, and generated SDKs.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to every `.proto` file under `microservices/*/contracts/`.

It applies to gRPC service definitions.

It applies to worker message contracts.

It applies to generated Rust, TypeScript, Kotlin, Swift, and Go clients.

It applies to protobuf evolution.

It applies to field reservations.

It applies to internal service-to-service contracts.

It applies to API-only capability tiers that use gRPC internally.

It does not replace OpenAPI for public REST.

It does not replace AsyncAPI for event contracts.

It does not authorize direct service calls when workflow is required.

## Normative Requirements

G-001. Every proto file MUST declare `syntax = "proto3";`.

G-002. Every proto file MUST declare a package.

G-003. Package names MUST use dot-separated canonical ids.

G-004. Package names MUST include version.

G-005. Every proto file MUST declare option metadata for generated packages when language requires it.

G-006. Every service MUST be named for a bounded context.

G-007. Every RPC MUST have stable name.

G-008. Every RPC MUST have request and response messages.

G-009. Every request that mutates state MUST include idempotency key.

G-010. Every request MUST include tenant context or explicit system posture.

G-011. Every request MUST include principal context or explicit system posture.

G-012. Every request carrying data MUST include data class.

G-013. Every mutating response SHOULD include audit event id.

G-014. Every RPC that invokes Cedar MUST declare Cedar action in comments or custom option.

G-015. Every RPC that starts workflow MUST declare workflow template id.

G-016. Every RPC that exposes ontology object MUST declare ontology id.

G-017. Every message field MUST have a stable tag.

G-018. Field tags MUST never be reused.

G-019. Removed fields MUST be reserved by tag and name.

G-020. Removed enum values MUST be reserved by number and name.

G-021. New fields MUST be additive.

G-022. Required semantics MUST be enforced at validation layer, not by proto3 required fields.

G-023. `oneof` MUST be used for mutually exclusive provider payloads.

G-024. `optional` MUST be used deliberately for presence.

G-025. Maps MUST be bounded by service validation.

G-026. Repeated fields MUST declare max cardinality in comments or options.

G-027. Bytes fields MUST declare encoding.

G-028. Timestamp fields MUST declare time source.

G-029. Money fields MUST not use float.

G-030. Decimal fields MUST use string or fixed scaled integer.

G-031. Enum zero value MUST be unknown or unspecified.

G-032. Enum values MUST be prefixed consistently.

G-033. Services MUST not expose database table names.

G-034. Services MUST not expose provider-specific fields outside adapter contracts.

G-035. Services MUST not bypass API surface separation.

G-036. Services MUST not bypass workflow when saga compensation is required.

G-037. Services MUST not bypass Cedar for tenant-scoped operations.

G-038. Services MUST not expose regulated fields without data-class annotation.

G-039. Proto comments MUST identify owner and ADR when contract is public.

G-040. Proto comments MUST identify deprecation and replacement when deprecated.

G-041. Generated code MUST not be edited by hand.

G-042. Generated code MUST identify generator version.

G-043. Generated SDKs MUST follow SDK layer rules.

G-044. Server implementations MUST live in grpc or app layers, not kernel.

G-045. Contract tests MUST run compatibility checks.

G-046. Contract tests MUST run generated client smoke tests.

G-047. Contract tests MUST run backward compatibility tests for published contracts.

G-048. Breaking changes MUST increment major version.

G-049. Minor additive changes MUST preserve old clients.

G-050. Deprecated fields MUST remain readable until sunset.

G-051. Proto files MUST NOT be named `schema.proto`.

G-052. Proto files MUST NOT mix unrelated bounded contexts.

G-053. Proto files MUST NOT use tag numbers casually.

G-054. Proto files MUST NOT reuse deleted tags.

G-055. Proto files MUST NOT use `float` for money.

G-056. Proto files MUST NOT use stringly typed JSON blobs for domain payloads.

G-057. Proto files MUST NOT hide tenant context in metadata only.

G-058. Proto files MUST NOT use provider-specific names in shared contracts.

G-059. Proto files MUST NOT skip examples for public SDKs.

G-060. Proto files MUST NOT rely on generation defaults as documented behavior.

## Worked Examples

### Example 1: Package and service

```proto
syntax = "proto3";

package oya.workflow_engine.state_machine.v1;

service WorkflowStateMachineService {
  rpc CompleteStep(CompleteStepRequest) returns (CompleteStepResponse);
}
```

This passes because package and service map to canonical ids.

### Example 2: Mutating request

```proto
message CompleteStepRequest {
  string tenant_id = 1;
  string principal_id = 2;
  string workflow_id = 3;
  string step_id = 4;
  string idempotency_key = 5;
  string data_class = 6;
}
```

This passes because tenant, principal, and idempotency are explicit.

### Example 3: Reserved field

```proto
message TenantGrant {
  reserved 7;
  reserved "legacy_product_module";
  string capability_tier = 1;
}
```

This passes because deleted field number and name cannot be reused.

### Example 4: Provider payload oneof

```proto
message ConnectorPayload {
  oneof provider_payload {
    SalesforcePayload salesforce = 10;
    NetsuitePayload netsuite = 11;
  }
}
```

This passes because provider variants are explicit.

### Example 5: Invalid money field

```proto
message Invoice {
  float total = 1;
}
```

This fails because money must not use floating point.

## Verification

Primary command:

```bash
oya gate validate proto3-authoring --scope microservices
```

Companion commands:

```bash
oya gate validate grpc-contract-compatibility --scope microservices
oya gate validate sdk-generation --scope microservices
```

The checker MUST parse every proto file.

The checker MUST verify proto3 syntax.

The checker MUST verify package version.

The checker MUST verify service naming.

The checker MUST verify tenant fields.

The checker MUST verify principal fields.

The checker MUST verify idempotency fields.

The checker MUST verify data-class fields.

The checker MUST verify audit event comments or options.

The checker MUST verify Cedar action comments or options.

The checker MUST verify workflow template comments or options.

The checker MUST verify reserved fields.

The checker MUST verify enum zero value.

The checker MUST verify removed tag reservations.

The checker MUST verify money field shape.

The checker MUST verify generated SDK smoke tests.

The checker MUST verify backward compatibility for published contracts.

The checker SHOULD emit descriptor set evidence.

The checker SHOULD emit language generation evidence.

## Common Anti-Patterns

Using proto2 syntax is an anti-pattern.

Using unversioned packages is an anti-pattern.

Using `schema.proto` is an anti-pattern.

Reusing deleted tag numbers is an anti-pattern.

Deleting fields without reserving tags is an anti-pattern.

Using float for money is an anti-pattern.

Using JSON blobs for domain payloads is an anti-pattern.

Hiding tenant id in gRPC metadata only is an anti-pattern.

Skipping idempotency on mutation is an anti-pattern.

Putting provider payloads in shared messages without oneof is an anti-pattern.

Generating SDKs without smoke tests is an anti-pattern.

Editing generated code manually is an anti-pattern.

Treating proto comments as optional for public contracts is an anti-pattern.

Using gRPC to bypass workflow is an anti-pattern.

Using gRPC to bypass Cedar is an anti-pattern.

## Cross-References

External authority: `https://protobuf.dev/programming-guides/proto3/`.

`docs/standards/openapi-3-2-authoring.md` binds REST contracts.

`docs/standards/asyncapi-3-1-authoring.md` binds async contracts.

`docs/standards/workflow-substrate-engine.md` binds workflow call discipline.

`docs/standards/api-surface-separation.md` binds internal/external separation.

`docs/standards/layer-enum-adr-0105.md` binds SDK and gRPC layers.

`docs/decisions/ADR-0705-product-protocol-live-apex.md` binds versioning.

`docs/decisions/ADR-0709-general-live-apex.md` binds tier contracts.

## Substance Bar Compliance Checklist

PROTO-SB-001. Verify `syntax = "proto3";`.

PROTO-SB-002. Verify package name.

PROTO-SB-003. Verify package version.

PROTO-SB-004. Verify language options.

PROTO-SB-005. Verify service name.

PROTO-SB-006. Verify RPC names.

PROTO-SB-007. Verify request messages.

PROTO-SB-008. Verify response messages.

PROTO-SB-009. Verify mutation idempotency key.

PROTO-SB-010. Verify tenant context.

PROTO-SB-011. Verify principal context.

PROTO-SB-012. Verify data class.

PROTO-SB-013. Verify audit event id.

PROTO-SB-014. Verify Cedar action annotation.

PROTO-SB-015. Verify workflow template annotation.

PROTO-SB-016. Verify ontology id annotation.

PROTO-SB-017. Verify stable field tags.

PROTO-SB-018. Verify removed field reservations.

PROTO-SB-019. Verify removed enum reservations.

PROTO-SB-020. Verify additive field change.

PROTO-SB-021. Verify validation layer for required semantics.

PROTO-SB-022. Verify oneof provider payload.

PROTO-SB-023. Verify optional presence rationale.

PROTO-SB-024. Verify map bounds.

PROTO-SB-025. Verify repeated field max cardinality.

PROTO-SB-026. Verify bytes encoding.

PROTO-SB-027. Verify timestamp source.

PROTO-SB-028. Verify money field shape.

PROTO-SB-029. Verify decimal field shape.

PROTO-SB-030. Verify enum zero value.

PROTO-SB-031. Verify enum prefixing.

PROTO-SB-032. Verify no database table exposure.

PROTO-SB-033. Verify no provider field leakage.

PROTO-SB-034. Verify API surface separation.

PROTO-SB-035. Verify workflow-required calls do not bypass workflow.

PROTO-SB-036. Verify Cedar-required calls do not bypass Cedar.

PROTO-SB-037. Verify regulated fields have data classes.

PROTO-SB-038. Verify public comments cite owner.

PROTO-SB-039. Verify deprecation replacement.

PROTO-SB-040. Verify generated code is not edited.

PROTO-SB-041. Check `oya.workflow_engine.state_machine.v1`.

PROTO-SB-042. Check `oya.tenancy.lifecycle.v1`.

PROTO-SB-043. Check `oya.policy_engine.cedar.v1`.

PROTO-SB-044. Check `oya.ontology.projection.v1`.

PROTO-SB-045. Check `oya.cloud.compute.vm.v1`.

PROTO-SB-046. Check `oya.messenger.delivery.v1`.

PROTO-SB-047. Check `oya.mail.message.v1`.

PROTO-SB-048. Check `oya.capability_tier.registry.v1`.

PROTO-SB-049. Check `CompleteStepRequest`.

PROTO-SB-050. Check `TenantGrant`.

PROTO-SB-051. Reject proto2 syntax.

PROTO-SB-052. Reject unversioned package.

PROTO-SB-053. Reject `schema.proto`.

PROTO-SB-054. Reject reused tag.

PROTO-SB-055. Reject deleted field without reservation.

PROTO-SB-056. Reject float money.

PROTO-SB-057. Reject JSON blob payload.

PROTO-SB-058. Reject hidden tenant metadata only.

PROTO-SB-059. Reject mutation without idempotency.

PROTO-SB-060. Reject generated code edits.

PROTO-SB-061. Emit proto file count.

PROTO-SB-062. Emit package count.

PROTO-SB-063. Emit service count.

PROTO-SB-064. Emit RPC count.

PROTO-SB-065. Emit message count.

PROTO-SB-066. Emit reserved tag count.

PROTO-SB-067. Emit enum count.

PROTO-SB-068. Emit oneof count.

PROTO-SB-069. Emit generated SDK count.

PROTO-SB-070. Emit compatibility test count.

PROTO-SB-071. Preserve proto3 syntax.

PROTO-SB-072. Preserve field tag immutability.

PROTO-SB-073. Preserve reserved deleted fields.

PROTO-SB-074. Preserve tenant context.

PROTO-SB-075. Preserve idempotency.

PROTO-SB-076. Preserve data-class labeling.

PROTO-SB-077. Preserve Cedar linkage.

PROTO-SB-078. Preserve Workflow linkage.

PROTO-SB-079. Preserve generated SDK compatibility.

PROTO-SB-080. Preserve major-version boundary for breaking changes.

## Extended Worked Example: Workflow gRPC Contract

```proto
syntax = "proto3";

package oyatie.workflow.v1;

option go_package = "github.com/oyatie/sdk-go/workflow/v1;workflowv1";

service WorkflowExecutionService {
  rpc StartWorkflowExecution(StartWorkflowExecutionRequest)
      returns (StartWorkflowExecutionResponse);

  rpc CancelWorkflowExecution(CancelWorkflowExecutionRequest)
      returns (CancelWorkflowExecutionResponse);
}

message StartWorkflowExecutionRequest {
  string tenant_id = 1;
  string workflow_template_id = 2;
  string idempotency_key = 3;
  string trace_id = 4;
  map<string, string> input_refs = 5;
}

message StartWorkflowExecutionResponse {
  string workflow_execution_id = 1;
  string state = 2;
  string accepted_at = 3;
}

message CancelWorkflowExecutionRequest {
  string tenant_id = 1;
  string workflow_execution_id = 2;
  string idempotency_key = 3;
  string trace_id = 4;
  string reason_code = 5;
}

message CancelWorkflowExecutionResponse {
  string workflow_execution_id = 1;
  string state = 2;
  string cancelled_at = 3;
}
```

## Extended Proto3 Matrix

| ID | Concern | Requirement | Example | Checker |
|---|---|---|---|---|
| PROTO-MAT-001 | Syntax | `proto3` | root field | `check-proto-syntax` |
| PROTO-MAT-002 | Package | versioned package | `oyatie.workflow.v1` | `check-proto-package` |
| PROTO-MAT-003 | Service | stable service name | `WorkflowExecutionService` | `check-proto-service` |
| PROTO-MAT-004 | RPC | stable RPC name | `StartWorkflowExecution` | `check-proto-rpc` |
| PROTO-MAT-005 | Field | no field reuse | field numbers | `check-proto-field-numbers` |
| PROTO-MAT-006 | Field | idempotency key on mutation | `idempotency_key` | `check-idempotency` |
| PROTO-MAT-007 | Field | trace id on boundary | `trace_id` | `check-trace-context` |
| PROTO-MAT-008 | Field | tenant id on tenant surface | `tenant_id` | `check-tenant-boundary` |
| PROTO-MAT-009 | Compatibility | reserved removed fields | `reserved` | `check-proto-compatibility` |
| PROTO-MAT-010 | Data class | option or manifest label | manifest | `check-data-class` |
| PROTO-MAT-011 | Cedar | RPC maps to action | policy map | `check-cedar-action-coverage` |
| PROTO-MAT-012 | Workflow | workflow RPC maps to template | workflow manifest | `check-workflow-linkage` |
| PROTO-MAT-013 | SDK | generated SDK compiles | SDK path | `check-proto-sdk` |
| PROTO-MAT-014 | Gateway | REST gateway parity if exposed | OpenAPI path | `check-contract-parity` |
| PROTO-MAT-015 | Errors | typed status mapping | error map | `check-grpc-error-map` |
| PROTO-MAT-016 | Examples | synthetic fixtures | fixtures | `check-example-safety` |
| PROTO-MAT-017 | Versioning | breaking change bumps package | package v2 | `check-contract-version` |
| PROTO-MAT-018 | Docs | ADR links exist | docs path | `check-doc-links` |
| PROTO-MAT-019 | Audit | mutating RPC emits event | audit map | `check-audit-emission` |
| PROTO-MAT-020 | Promote | checker output in evidence | VCS bundle | `oya-vcs-admission` |

## Extended Proto Evidence Ledger

PROTO-EVID-001. Record proto file path.

PROTO-EVID-002. Record syntax version.

PROTO-EVID-003. Record package name.

PROTO-EVID-004. Record package major version.

PROTO-EVID-005. Record service names.

PROTO-EVID-006. Record RPC names.

PROTO-EVID-007. Record message names.

PROTO-EVID-008. Record field-number map.

PROTO-EVID-009. Record reserved field numbers.

PROTO-EVID-010. Record reserved field names.

PROTO-EVID-011. Record idempotency field coverage.

PROTO-EVID-012. Record trace-id field coverage.

PROTO-EVID-013. Record tenant-id field coverage.

PROTO-EVID-014. Record data-class label coverage.

PROTO-EVID-015. Record Cedar action mapping.

PROTO-EVID-016. Record workflow template mapping.

PROTO-EVID-017. Record gRPC status mapping.

PROTO-EVID-018. Record generated SDK package.

PROTO-EVID-019. Record generated SDK compile result.

PROTO-EVID-020. Record REST gateway parity result.

PROTO-EVID-021. Record synthetic fixture count.

PROTO-EVID-022. Record unsafe fixture findings.

PROTO-EVID-023. Record compatibility diff result.

PROTO-EVID-024. Record breaking-change count.

PROTO-EVID-025. Record audit-emission mapping.

PROTO-EVID-026. Record checker crate version.

PROTO-EVID-027. Record VCS changeset id.

PROTO-EVID-028. Record promote bundle id.

## Extended Proto Anti-Patterns

PROTO-APX-001. Reuse deleted field number.

PROTO-APX-002. Remove field without reserving number and name.

PROTO-APX-003. Mutation lacks idempotency key.

PROTO-APX-004. Boundary message lacks trace id.

PROTO-APX-005. Tenant RPC lacks tenant id.

PROTO-APX-006. Breaking change remains in v1 package.

PROTO-APX-007. Generated SDK compile is skipped.

PROTO-APX-008. gRPC errors are unmapped strings.

PROTO-APX-009. Proto contract diverges from OpenAPI gateway.

PROTO-APX-010. Promote evidence omits proto diff output.

## Extended Promotion Review Checklist

PROTO-PROMOTE-001. Proto file path is stable.

PROTO-PROMOTE-002. Syntax version is proto3.

PROTO-PROMOTE-003. Package name is versioned.

PROTO-PROMOTE-004. Package major version is recorded.

PROTO-PROMOTE-005. Service names are stable.

PROTO-PROMOTE-006. RPC names are stable.

PROTO-PROMOTE-007. Message names are stable.

PROTO-PROMOTE-008. Field-number map is recorded.

PROTO-PROMOTE-009. Reserved field numbers are recorded.

PROTO-PROMOTE-010. Reserved field names are recorded.

PROTO-PROMOTE-011. Idempotency field coverage is complete.

PROTO-PROMOTE-012. Trace-id field coverage is complete.

PROTO-PROMOTE-013. Tenant-id field coverage is complete.

PROTO-PROMOTE-014. Data-class label coverage is complete.

PROTO-PROMOTE-015. Cedar action mapping is attached.

PROTO-PROMOTE-016. Workflow template mapping is attached.

PROTO-PROMOTE-017. gRPC status mapping is attached.

PROTO-PROMOTE-018. Generated SDK package is recorded.

PROTO-PROMOTE-019. Generated SDK compile result is attached.

PROTO-PROMOTE-020. REST gateway parity result is attached.

PROTO-PROMOTE-021. Synthetic fixture count is recorded.

PROTO-PROMOTE-022. Unsafe fixture findings are zero.

PROTO-PROMOTE-023. Compatibility diff result is attached.

PROTO-PROMOTE-024. Breaking-change count is zero or major bump exists.

PROTO-PROMOTE-025. Audit-emission mapping is attached.

PROTO-PROMOTE-026. Checker crate version is recorded.

PROTO-PROMOTE-027. VCS changeset id is recorded.

PROTO-PROMOTE-028. Promote bundle id is recorded.

PROTO-PROMOTE-029. Field number reuse count is zero.

PROTO-PROMOTE-030. Removed fields are reserved.

PROTO-PROMOTE-031. Removed enum values are reserved.

PROTO-PROMOTE-032. Mutation RPCs carry idempotency key.

PROTO-PROMOTE-033. Boundary RPCs carry trace id.

PROTO-PROMOTE-034. Tenant RPCs carry tenant id.

PROTO-PROMOTE-035. Major version changes use a new package.

PROTO-PROMOTE-036. Deprecated fields have sunset notes.

PROTO-PROMOTE-037. Optional fields have compatibility rationale.

PROTO-PROMOTE-038. Maps are not used for typed domain objects.

PROTO-PROMOTE-039. Bytes fields have encoding notes.

PROTO-PROMOTE-040. Timestamps use documented representation.

PROTO-PROMOTE-041. Money values avoid floating point.

PROTO-PROMOTE-042. Pagination messages align with OpenAPI cursor shape.

PROTO-PROMOTE-043. Error details align with gRPC status mapping.

PROTO-PROMOTE-044. Generated Rust SDK compiles.

PROTO-PROMOTE-045. Generated TypeScript SDK compiles.

PROTO-PROMOTE-046. Generated Go SDK compiles when configured.

PROTO-PROMOTE-047. Buf lint output is attached.

PROTO-PROMOTE-048. Buf breaking output is attached.

PROTO-PROMOTE-049. Contract parity output is attached.

PROTO-PROMOTE-050. Promotion evidence includes Proto checker output.

## Extended Proto Residual-Risk Register

PROTO-RISK-001. Residual risk: field number reused after deletion; mitigation is reserved-number checker.

PROTO-RISK-002. Residual risk: generated SDK accepts zero-value ambiguity; mitigation is explicit presence review.

PROTO-RISK-003. Residual risk: money field uses float; mitigation is fixed integer minor-unit rule.

PROTO-RISK-004. Residual risk: timestamp representation drifts from OpenAPI; mitigation is contract parity checker.

PROTO-RISK-005. Residual risk: gRPC status hides Cedar denial; mitigation is Cedar denial status mapping.

PROTO-RISK-006. Residual risk: REST gateway strips trace id; mitigation is gateway parity fixture.

PROTO-RISK-007. Residual risk: streaming RPC omits backpressure semantics; mitigation is stream rubric review.

PROTO-RISK-008. Residual risk: map field replaces typed domain object; mitigation is schema design review.

PROTO-RISK-009. Residual risk: enum default has business meaning; mitigation is unspecified-zero-value rule.

PROTO-RISK-010. Residual risk: client SDK pins stale package version; mitigation is generated SDK compatibility test.

PROTO-RISK-011. Residual risk: tenant id is trusted from body instead of identity context; mitigation is tenant-boundary gate.

PROTO-RISK-012. Residual risk: idempotency key differs from OpenAPI gateway key; mitigation is idempotency parity check.

PROTO-RISK-013. Residual risk: audit event mapping omits failed calls; mitigation is error-path audit fixture.

PROTO-RISK-014. Residual risk: breaking package bump lacks migration guide; mitigation is contract migration checklist.

PROTO-RISK-015. Residual risk: consumer compiles generated SDK but lacks behavior fixture; mitigation is consumer contract test.

PROTO-RISK-016. Residual risk: proto comments become stale authority; mitigation is ADR and standard cross-reference check.

PROTO-RISK-017. Residual risk: binary compatibility passes while semantic compatibility fails; mitigation is semantic fixture.

PROTO-RISK-018. Residual risk: reserved names omit JSON aliases; mitigation is JSON-name compatibility check.

PROTO-RISK-019. Residual risk: package name omits owning domain; mitigation is naming BNF checker.

PROTO-RISK-020. Residual risk: promote bundle omits Buf output; mitigation is VCS evidence gate.
