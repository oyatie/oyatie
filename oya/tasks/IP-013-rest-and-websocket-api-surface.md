---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-013-rest-and-websocket-api-surface
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [openapi-lint, asyncapi-lint, protoc-lint, oya-governance-contract-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: REST + WebSocket API surface — OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3 wiring

## Intent

Wire the public API surface across all 7 BCs to the contracts authored
in `contracts/openapi/tasks.yaml` (OpenAPI 3.2.0), `contracts/asyncapi/
tasks-events.yaml` (AsyncAPI 3.1.0), and `contracts/proto/tasks.proto`
(proto3). Each BC's `-rest` crate registers handlers per the OpenAPI
paths; each BC's `-worker` crate publishes events per AsyncAPI; the
gRPC peer surface (workflow-engine bridge per ADR-TASKS-0005) speaks
the proto3 services.

WebSocket gateway lives on `view-engine-rest` (port 8443) per IP-010.
REST endpoints unified behind `task-store-rest` (port 8080) at the
public edge; per-BC sub-routers mounted under `/v1/{tasks,projects,...}`.

`contract-coverage` lane refuses to merge if any `-rest` route is
absent from the OpenAPI surface OR any worker event is absent from the
AsyncAPI surface.

## ChangeSet boundary

`task-store-rest`, `project-list-rest`, `view-engine-rest`,
`dependency-graph-rest`, `search-index-rest`, plus a thin gRPC peer
crate `task-store-rest` (gRPC tonic server). The contract lint tests
land at `microservices/tasks/tests/contract-lint/`.

## Crate Naming

n/a — modifies existing `-rest` crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-*-rest/src/router.rs` | created/replaced | route registration |
| `microservices/tasks/src/oya-tasks-task-store-rest/src/grpc.rs` | created | tonic server impl |
| `microservices/tasks/tests/contract-lint/openapi.rs` | created | lint test |
| `microservices/tasks/tests/contract-lint/asyncapi.rs` | created | lint test |
| `microservices/tasks/tests/contract-lint/proto.rs` | created | lint test |

## Acceptance Gates

```bash
npx @apidevtools/swagger-cli validate microservices/tasks/contracts/openapi/tasks.yaml
npx @asyncapi/cli validate microservices/tasks/contracts/asyncapi/tasks-events.yaml
protoc --proto_path=microservices/tasks/contracts/proto --rust_out=/tmp microservices/tasks/contracts/proto/tasks.proto
cargo test -p oya-tasks-task-store-rest
buck2 build //:quality-lane-registry-authority-check # lane=contract-coverage --microservice tasks
```

## Test Plan

- Every BC's REST surface is covered by an OpenAPI path.
- Every PRD §"Workflow events produced" row is covered by an AsyncAPI
  channel.
- Every BC's gRPC surface is covered by a proto3 service.
- WebSocket gateway smoke: open connection; subscribe to project;
  receive `BoardReordered` event after a reorder REST call.

## Halt Conditions

- Coverage gap detected — refuse to ship; this is a hyperscaler-bar
  contract-completeness requirement.

## Next IP

[`IP-014-ai-assist-bounds-and-eu-ai-act.md`](IP-014-ai-assist-bounds-and-eu-ai-act.md)

## References

- OpenAPI 3.2.0 — `spec.openapis.org/oas/v3.2.0`.
- AsyncAPI 3.1.0 — `www.asyncapi.com/docs/reference/specification/v3.1.0`.
- proto3 — `protobuf.dev/programming-guides/proto3/`.
