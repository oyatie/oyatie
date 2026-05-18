---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-011-contracts-openapi-asyncapi-proto
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [openapi-lint, asyncapi-lint, protoc-lint, oya-governance-contract-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: contracts — OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3 finalisation

## Intent

Finalise the three wire-protocol contracts. Existing files
(`contracts/openapi/calendar.yaml`, `contracts/asyncapi/calendar-
events.yaml`, `contracts/proto/calendar.proto`) ship at OpenAPI 3.1.0;
this IP bumps to OpenAPI 3.2.0 + AsyncAPI 3.1.0 + ensures proto3
coverage for every BC.

## ChangeSet boundary

3 contracts files (modify in place) + 1 lint-test crate.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/contracts/openapi/calendar.yaml` | bump | OpenAPI 3.1.0 → 3.2.0; cover all 6 BCs; add JMAP Calendars stub (M04 placeholder) |
| `microservices/calendar/contracts/asyncapi/calendar-events.yaml` | bump | AsyncAPI 2.x → 3.1.0; cover all PRD §"Workflow events produced" rows |
| `microservices/calendar/contracts/proto/calendar.proto` | extend | cover all 6 BCs; add streaming endpoints for event-lifecycle subscription |
| `microservices/calendar/tests/contract-lint/` | create | OpenAPI/AsyncAPI/proto lint tests |

## Acceptance Gates

```bash
npx @apidevtools/swagger-cli validate microservices/calendar/contracts/openapi/calendar.yaml
npx @asyncapi/cli validate microservices/calendar/contracts/asyncapi/calendar-events.yaml
protoc --proto_path=microservices/calendar/contracts/proto --rust_out=/tmp microservices/calendar/contracts/proto/calendar.proto
cargo run -p oya-dev-cli -- gate validate contract-coverage --microservice calendar
```

## Test Plan

- Every BC's `-rest` crate's surface is covered by an OpenAPI path.
- Every PRD §"Workflow events produced" event is covered by an
  AsyncAPI channel.
- Every BC's gRPC surface is covered by a proto3 service.

## Halt Conditions

- Any BC's REST surface missing from OpenAPI — block.
- Any Workflow event missing from AsyncAPI — block.

## Next IP

[`IP-012-cedar-policies-and-data-residency.md`](IP-012-cedar-policies-and-data-residency.md)

## References

- OpenAPI 3.2.0 spec — `spec.openapis.org/oas/v3.2.0`.
- AsyncAPI 3.1.0 spec — `www.asyncapi.com/docs/reference/specification/v3.1.0`.
- proto3 spec — `protobuf.dev/programming-guides/proto3/`.
