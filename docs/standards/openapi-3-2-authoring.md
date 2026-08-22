---
doc_class: Standard
title: OpenAPI 3.2 Authoring Standard
status: Accepted
date: 2026-05-20
owner: council-architecture + axis-api-gateway
related_oyatie_adrs:
  - ADR-0037
  - ADR-0131
  - ADR-0145
  - ADR-0177
  - ADR-0258
enforced_by:
  - governance-openapi-3-2-authoring
  - governance-openapi-rest-route-parity
  - governance-api-versioning
canonical_paths:
  - contracts/*.openapi.yaml
  - microservices/*/contracts/*.openapi.yaml
  - docs/standards/api-design.md
  - docs/standards/api-surface-separation.md
external_reference:
  - https://spec.openapis.org/oas/v3.2.0.html
---

# OpenAPI 3.2 Authoring Standard

OpenAPI 3.2.0 is the canonical REST contract format for Oyatie public and
internal HTTP APIs. This standard defines the Oyatie profile for authoring,
versioning, validating, testing, generating SDKs from, and governing OpenAPI
contracts. The OpenAPI Initiative specification is the external authority; this
standard adds tenant, principal, Cedar, data-class, idempotency, audit, SLO, and
microservice layout requirements.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to every `*.openapi.yaml` file.

It applies to public REST APIs.

It applies to internal REST APIs.

It applies to generated SDK inputs.

It applies to contract tests.

It applies to route parity checks.

It applies to API deprecation.

It applies to API versioning.

It applies to OpenAPI examples.

It does not cover AsyncAPI events.

It does not cover proto3 service definitions.

It does not override API surface separation rules.

## Normative Requirements

OAS-001. Every REST contract MUST declare OpenAPI version 3.2.0.

OAS-002. Every REST contract MUST declare `info.title`.

OAS-003. Every REST contract MUST declare `info.version`.

OAS-004. Every REST contract MUST declare owning microservice.

OAS-005. Every REST contract MUST declare server posture or environment-neutral server variables.

OAS-006. Every path MUST be versioned or explicitly internal.

OAS-007. Every operation MUST have `operationId`.

OAS-008. Every operationId MUST be stable.

OAS-009. Every operation MUST declare tags.

OAS-010. Every operation MUST declare security requirements.

OAS-011. Every mutating operation MUST require `Idempotency-Key`.

OAS-012. `Idempotency-Key` MUST have `minLength: 1`.

OAS-013. Every operation MUST require or derive request id.

OAS-014. Every tenant-scoped operation MUST include tenant context.

OAS-015. Every principal-scoped operation MUST include principal context.

OAS-016. Every data-bearing schema MUST declare data class.

OAS-017. Every regulated operation MUST declare compliance pack overlays.

OAS-018. Every response MUST declare success status.

OAS-019. Every operation MUST declare error responses.

OAS-020. Every error response MUST use canonical error schema.

OAS-021. Every Cedar denial MUST use canonical Cedar denial schema.

OAS-022. Every validation failure MUST use canonical validation error schema.

OAS-023. Every pagination operation MUST use cursor pagination unless an ADR permits otherwise.

OAS-024. Every list operation MUST declare max page size.

OAS-025. Every list operation MUST declare sort stability.

OAS-026. Every create operation MUST declare idempotency behavior.

OAS-027. Every update operation MUST declare concurrency behavior.

OAS-028. Every delete operation MUST declare soft/hard delete posture.

OAS-029. Every destructive operation MUST declare acknowledgement requirements.

OAS-030. Every operation that emits audit event MUST declare event class.

OAS-031. Every operation that starts workflow MUST declare workflow template id.

OAS-032. Every operation that exposes ontology object MUST declare ontology id.

OAS-033. Every operation that consumes capability tier MUST declare tier id.

OAS-034. Every operation MUST include examples.

OAS-035. Every example MUST be schema-valid.

OAS-036. Every example MUST use synthetic tenant data.

OAS-037. Every schema MUST be reusable when shared by more than one operation.

OAS-038. Every enum MUST declare evolution policy.

OAS-039. Every nullable field MUST be intentional.

OAS-040. Every deprecated field MUST declare sunset.

OAS-041. Every deprecated operation MUST declare replacement.

OAS-042. Every breaking change MUST increment major version.

OAS-043. Every non-breaking additive change MUST preserve old clients.

OAS-044. Every public operation MUST declare rate-limit behavior.

OAS-045. Every public operation MUST declare authentication class.

OAS-046. Every public operation MUST declare authorization action.

OAS-047. Every operation SHOULD declare SLO linkage.

OAS-048. Every operation SHOULD declare audit event linkage.

OAS-049. Every operation SHOULD declare runbook linkage for failure modes.

OAS-050. Every contract MUST pass route parity against implemented routes.

OAS-051. OpenAPI files MUST NOT be named `api.yaml`.

OAS-052. OpenAPI files MUST NOT mix unrelated bounded contexts.

OAS-053. OpenAPI files MUST NOT omit error schemas.

OAS-054. OpenAPI files MUST NOT use free-form objects for domain payloads.

OAS-055. OpenAPI files MUST NOT use real tenant data in examples.

OAS-056. OpenAPI files MUST NOT omit idempotency on mutation.

OAS-057. OpenAPI files MUST NOT use unbounded page size.

OAS-058. OpenAPI files MUST NOT bypass API surface separation.

OAS-059. OpenAPI files MUST NOT duplicate proto contracts without ownership.

OAS-060. OpenAPI files MUST NOT use implementation-only error strings as contract.

## Worked Examples

### Example 1: Mutating route

```yaml
openapi: 3.2.0
info:
  title: Workflow Engine API
  version: 1.0.0
paths:
  /v1/workflows/{workflow_id}/steps/{step_id}:complete:
    post:
      operationId: completeWorkflowStep
      parameters:
        - name: Idempotency-Key
          in: header
          required: true
          schema:
            type: string
            minLength: 1
      responses:
        "200":
          description: Step completed
```

This passes because mutation requires idempotency.

### Example 2: Cedar denial

```yaml
components:
  schemas:
    CedarDenyResponse:
      type: object
      required: [error, policy_decision_id]
      properties:
        error:
          const: cedar_denied
        policy_decision_id:
          type: string
```

This passes because policy denial is typed and auditable.

### Example 3: Cursor list

```yaml
parameters:
  - name: page_cursor
    in: query
    schema:
      type: string
  - name: page_size
    in: query
    schema:
      type: integer
      minimum: 1
      maximum: 100
```

This passes because page size is bounded.

### Example 4: Invalid mutation

```yaml
post:
  operationId: updateThing
  parameters: []
```

This fails because `Idempotency-Key` is missing.

### Example 5: Capability tier route

```yaml
x-oyatie-capability-tier: workflow-engine.approval-routing.professional
x-oyatie-audit-event: EVT-WORKFLOW-APPROVAL-APPROVED-V1
x-oyatie-cedar-action: Action::"ApproveWorkflowStep"
```

This passes because tier, audit, and authorization are explicit.

## Verification

Primary command:

```bash
oya gate validate openapi-3-2-authoring --scope contracts
```

Companion commands:

```bash
oya gate validate openapi-rest-route-parity --scope crates
oya gate validate api-versioning --scope contracts
oya doc openapi
```

The checker MUST parse all OpenAPI files.

The checker MUST validate version.

The checker MUST validate operation ids.

The checker MUST validate idempotency keys.

The checker MUST validate request id handling.

The checker MUST validate tenant context.

The checker MUST validate data-class fields.

The checker MUST validate error schemas.

The checker MUST validate Cedar denial schemas.

The checker MUST validate pagination.

The checker MUST validate examples.

The checker MUST validate deprecation metadata.

The checker MUST validate versioning metadata.

The checker MUST validate route parity.

The checker MUST validate capability tier extensions.

The checker MUST validate audit event extensions.

The checker MUST validate workflow extensions.

The checker SHOULD generate contract fixtures.

The checker SHOULD compare SDK generation output.

## Common Anti-Patterns

Using OpenAPI 3.0 or 3.1 for new contracts is an anti-pattern.

Naming the file `api.yaml` is an anti-pattern.

Skipping `Idempotency-Key` on mutation is an anti-pattern.

Using unbounded page size is an anti-pattern.

Returning raw strings as errors is an anti-pattern.

Omitting Cedar denial response is an anti-pattern.

Omitting examples is an anti-pattern.

Using real tenant examples is an anti-pattern.

Mixing public and internal routes without tags is an anti-pattern.

Changing field type without major version is an anti-pattern.

Using `nullable` without rationale is an anti-pattern.

Copying proto messages into OpenAPI without ownership is an anti-pattern.

Using operation ids that change with code refactors is an anti-pattern.

Treating route implementation as contract authority is an anti-pattern.

Treating generated SDK breakage as acceptable drift is an anti-pattern.

## Cross-References

External authority: `https://spec.openapis.org/oas/v3.2.0.html`.

`docs/standards/api-design.md` binds general API design.

`docs/standards/api-surface-separation.md` binds internal/external separation.

`docs/standards/cursor-pagination-canonical.md` binds pagination.

`docs/standards/idempotency-keys-canonical.md` binds idempotency.

`docs/standards/error-handling.md` binds error schema.

`docs/standards/asyncapi-3-1-authoring.md` binds async contracts.

`docs/standards/proto3-authoring.md` binds gRPC contracts.

`docs/decisions/ADR-0705-product-protocol-live-apex.md` binds API versioning.

## Substance Bar Compliance Checklist

OAS-SB-001. Verify OpenAPI version 3.2.0.

OAS-SB-002. Verify info title.

OAS-SB-003. Verify info version.

OAS-SB-004. Verify owning microservice extension.

OAS-SB-005. Verify server posture.

OAS-SB-006. Verify path versioning.

OAS-SB-007. Verify operationId.

OAS-SB-008. Verify operation tags.

OAS-SB-009. Verify security requirements.

OAS-SB-010. Verify idempotency header on mutation.

OAS-SB-011. Verify `Idempotency-Key` minLength.

OAS-SB-012. Verify request id.

OAS-SB-013. Verify tenant context.

OAS-SB-014. Verify principal context.

OAS-SB-015. Verify data class.

OAS-SB-016. Verify compliance pack overlay.

OAS-SB-017. Verify success responses.

OAS-SB-018. Verify error responses.

OAS-SB-019. Verify canonical error schema.

OAS-SB-020. Verify Cedar denial schema.

OAS-SB-021. Verify validation error schema.

OAS-SB-022. Verify cursor pagination.

OAS-SB-023. Verify max page size.

OAS-SB-024. Verify sort stability.

OAS-SB-025. Verify create idempotency.

OAS-SB-026. Verify update concurrency.

OAS-SB-027. Verify delete posture.

OAS-SB-028. Verify destructive acknowledgement.

OAS-SB-029. Verify audit event extension.

OAS-SB-030. Verify workflow extension.

OAS-SB-031. Verify ontology extension.

OAS-SB-032. Verify capability tier extension.

OAS-SB-033. Verify examples.

OAS-SB-034. Verify synthetic tenant data.

OAS-SB-035. Verify reusable schemas.

OAS-SB-036. Verify enum evolution.

OAS-SB-037. Verify nullable rationale.

OAS-SB-038. Verify deprecation sunset.

OAS-SB-039. Verify replacement operation.

OAS-SB-040. Verify route parity.

OAS-SB-041. Check `workflow-engine` step complete route.

OAS-SB-042. Check `tenancy` tenant create route.

OAS-SB-043. Check `policy-engine` Cedar publish route.

OAS-SB-044. Check `cloud-compute` VM route.

OAS-SB-045. Check `cloud-storage` object route.

OAS-SB-046. Check `messenger` delivery route.

OAS-SB-047. Check `mail` message route.

OAS-SB-048. Check `ontology` object route.

OAS-SB-049. Check `observability` audit route.

OAS-SB-050. Check `regional-pack` bind route.

OAS-SB-051. Reject OpenAPI 3.0 for new contract.

OAS-SB-052. Reject `api.yaml`.

OAS-SB-053. Reject mutation without idempotency.

OAS-SB-054. Reject unbounded page size.

OAS-SB-055. Reject raw string error.

OAS-SB-056. Reject missing Cedar denial schema.

OAS-SB-057. Reject real tenant examples.

OAS-SB-058. Reject field type breaking change without major bump.

OAS-SB-059. Reject route implementation not in contract.

OAS-SB-060. Reject contract route not implemented.

OAS-SB-061. Emit OpenAPI file count.

OAS-SB-062. Emit path count.

OAS-SB-063. Emit operation count.

OAS-SB-064. Emit schema count.

OAS-SB-065. Emit example count.

OAS-SB-066. Emit idempotency coverage count.

OAS-SB-067. Emit pagination coverage count.

OAS-SB-068. Emit Cedar denial coverage count.

OAS-SB-069. Emit route parity count.

OAS-SB-070. Emit SDK generation count.

OAS-SB-071. Preserve OpenAPI as contract authority.

OAS-SB-072. Preserve route parity.

OAS-SB-073. Preserve idempotency.

OAS-SB-074. Preserve request id propagation.

OAS-SB-075. Preserve tenant context.

OAS-SB-076. Preserve data-class labels.

OAS-SB-077. Preserve Cedar denial shape.

OAS-SB-078. Preserve cursor pagination.

OAS-SB-079. Preserve API versioning.

OAS-SB-080. Preserve generated SDK compatibility.

## Extended Worked Example: Tenant Registry REST Contract

```yaml
openapi: 3.2.0
info:
  title: Tenant Registry API
  version: 1.0.0
  x-oyatie-standard: docs/standards/openapi-3-2-authoring.md
  x-oyatie-adrs:
    - ADR-0145
    - ADR-0258
    - ADR-0313
servers:
  - url: https://api.oyatie.example/tenancy/v1
paths:
  /tenants/{tenant_id}/capability-ceiling:
    patch:
      operationId: tenantCapabilityCeilingUpdateV1
      summary: Update a tenant capability ceiling.
      parameters:
        - name: tenant_id
          in: path
          required: true
          schema:
            type: string
        - name: Idempotency-Key
          in: header
          required: true
          schema:
            type: string
        - name: X-Request-Id
          in: header
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/TenantCapabilityCeilingUpdateRequestV1'
      responses:
        '200':
          description: Capability ceiling updated.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TenantCapabilityCeilingV1'
        '403':
          description: Cedar denied the request.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OyatieProblemV1'
components:
  schemas:
    TenantCapabilityCeilingUpdateRequestV1:
      type: object
      required:
        - capability_ceiling
      properties:
        capability_ceiling:
          type: integer
          minimum: 0
          maximum: 4
    TenantCapabilityCeilingV1:
      type: object
      required:
        - tenant_id
        - capability_ceiling
        - updated_at
      properties:
        tenant_id:
          type: string
        capability_ceiling:
          type: integer
        updated_at:
          type: string
          format: date-time
    OyatieProblemV1:
      type: object
      required:
        - type
        - title
        - status
        - trace_id
      properties:
        type:
          type: string
        title:
          type: string
        status:
          type: integer
        trace_id:
          type: string
```

## Extended OpenAPI Matrix

| ID | Concern | Requirement | Example | Checker |
|---|---|---|---|---|
| OAS-MAT-001 | Version | `openapi: 3.2.0` | root field | `check-openapi-version` |
| OAS-MAT-002 | Info | standard link | `x-oyatie-standard` | `check-contract-links` |
| OAS-MAT-003 | Info | ADR links | `x-oyatie-adrs` | `check-adr-links` |
| OAS-MAT-004 | Operation | stable operation id | `tenantCapabilityCeilingUpdateV1` | `check-operation-ids` |
| OAS-MAT-005 | Idempotency | mutation header required | `Idempotency-Key` | `check-idempotency` |
| OAS-MAT-006 | Request id | request header required | `X-Request-Id` | `check-request-id` |
| OAS-MAT-007 | Errors | problem schema | `OyatieProblemV1` | `check-error-schema` |
| OAS-MAT-008 | Cedar | denial response documented | `403` | `check-cedar-denial-response` |
| OAS-MAT-009 | Pagination | cursor shape for list | `cursor` | `check-pagination` |
| OAS-MAT-010 | Data class | schema extension | `x-oyatie-data-class` | `check-data-class` |
| OAS-MAT-011 | Tenant | tenant path/header explicit | `tenant_id` | `check-tenant-boundary` |
| OAS-MAT-012 | Compatibility | diff run | schema diff | `check-openapi-compatibility` |
| OAS-MAT-013 | SDK | generated client compiles | SDK path | `check-sdk-generation` |
| OAS-MAT-014 | Route parity | handler exists | service route | `check-route-parity` |
| OAS-MAT-015 | Examples | examples are synthetic | examples | `check-example-safety` |
| OAS-MAT-016 | Security | security schemes declared | components | `check-security-schemes` |
| OAS-MAT-017 | Versioning | breaking change bumps major | info.version | `check-contract-version` |
| OAS-MAT-018 | Docs | standard cross-reference | docs path | `check-doc-links` |
| OAS-MAT-019 | Audit | mutating operation emits event | audit map | `check-audit-emission` |
| OAS-MAT-020 | Promote | checker output in evidence | VCS bundle | `oya-vcs-admission` |

## Extended OpenAPI Evidence Ledger

OAS-EVID-001. Record OpenAPI file path.

OAS-EVID-002. Record OpenAPI version.

OAS-EVID-003. Record contract title.

OAS-EVID-004. Record contract semantic version.

OAS-EVID-005. Record owning µservice.

OAS-EVID-006. Record related ADR ids.

OAS-EVID-007. Record server URLs.

OAS-EVID-008. Record path count.

OAS-EVID-009. Record operation count.

OAS-EVID-010. Record schema count.

OAS-EVID-011. Record operation ids.

OAS-EVID-012. Record mutation operation ids.

OAS-EVID-013. Record idempotency header coverage.

OAS-EVID-014. Record request-id header coverage.

OAS-EVID-015. Record Cedar denial response coverage.

OAS-EVID-016. Record problem schema coverage.

OAS-EVID-017. Record pagination coverage.

OAS-EVID-018. Record data-class extension coverage.

OAS-EVID-019. Record tenant-boundary coverage.

OAS-EVID-020. Record security scheme coverage.

OAS-EVID-021. Record synthetic example count.

OAS-EVID-022. Record unsafe example findings.

OAS-EVID-023. Record schema diff result.

OAS-EVID-024. Record breaking-change count.

OAS-EVID-025. Record route parity result.

OAS-EVID-026. Record generated SDK package.

OAS-EVID-027. Record generated SDK compile result.

OAS-EVID-028. Record audit-emission mapping result.

OAS-EVID-029. Record checker crate version.

OAS-EVID-030. Record VCS changeset id.

OAS-EVID-031. Record promote bundle id.

## Extended OpenAPI Anti-Patterns

OAS-APX-001. Mutation lacks idempotency key.

OAS-APX-002. Error response is raw string.

OAS-APX-003. Operation id changes without compatibility alias.

OAS-APX-004. Contract route has no implementation.

OAS-APX-005. Implementation route has no contract.

OAS-APX-006. Example contains real tenant id.

OAS-APX-007. List endpoint lacks cursor pagination.

OAS-APX-008. Cedar denial is documented as generic 500.

OAS-APX-009. Breaking schema change keeps same major version.

OAS-APX-010. SDK generation is skipped before promote.

## Extended Promotion Review Checklist

OAS-PROMOTE-001. OpenAPI file path is stable.

OAS-PROMOTE-002. OpenAPI version is 3.2.0.

OAS-PROMOTE-003. Contract title is recorded.

OAS-PROMOTE-004. Contract semantic version is recorded.

OAS-PROMOTE-005. Owning µservice is recorded.

OAS-PROMOTE-006. Related ADR ids are recorded.

OAS-PROMOTE-007. Server URLs are recorded.

OAS-PROMOTE-008. Path count is recorded.

OAS-PROMOTE-009. Operation count is recorded.

OAS-PROMOTE-010. Schema count is recorded.

OAS-PROMOTE-011. Operation ids are stable.

OAS-PROMOTE-012. Mutation operation ids are listed.

OAS-PROMOTE-013. Idempotency header coverage is complete.

OAS-PROMOTE-014. Request-id header coverage is complete.

OAS-PROMOTE-015. Cedar denial response coverage is complete.

OAS-PROMOTE-016. Problem schema coverage is complete.

OAS-PROMOTE-017. Pagination coverage is complete.

OAS-PROMOTE-018. Data-class extension coverage is complete.

OAS-PROMOTE-019. Tenant-boundary coverage is complete.

OAS-PROMOTE-020. Security scheme coverage is complete.

OAS-PROMOTE-021. Synthetic examples are safe.

OAS-PROMOTE-022. Schema diff result is attached.

OAS-PROMOTE-023. Breaking-change count is zero or major bump exists.

OAS-PROMOTE-024. Route parity result is attached.

OAS-PROMOTE-025. Generated SDK package compiles.

OAS-PROMOTE-026. Audit-emission mapping is attached.

OAS-PROMOTE-027. Checker crate version is recorded.

OAS-PROMOTE-028. VCS changeset id is recorded.

OAS-PROMOTE-029. Promote bundle id is recorded.

OAS-PROMOTE-030. Promotion evidence includes OpenAPI checker output.
