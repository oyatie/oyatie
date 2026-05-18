---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-013-contracts-openapi-asyncapi-proto
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet
acceptance_lanes: [openapi-schema-validate, asyncapi-schema-validate, proto-lint]
---

# IP-013: public contracts (OpenAPI 3.2 + AsyncAPI 3.1 + Protobuf v3)

## Intent

Author the public contracts:
- `contracts/openapi/meet.yaml` — REST surface (room/instance/participant/recording/transcript/webinar/egress).
- `contracts/asyncapi/meet-events.yaml` — WebSocket + Workflow event surface.
- `contracts/proto/meet.proto` — gRPC peer for clients preferring gRPC.

These are the SDK-generation source-of-truth + tenant-facing schema documentation surface.

## Concrete File Targets

| Path | Action |
|---|---|
| `contracts/openapi/meet.yaml` | create — OpenAPI 3.2 |
| `contracts/asyncapi/meet-events.yaml` | create — AsyncAPI 3.1 |
| `contracts/proto/meet.proto` | create — proto3 + grpc service defs |

## Acceptance Gates

```bash
spectral lint contracts/openapi/meet.yaml --ruleset .spectral.yaml
asyncapi-validator contracts/asyncapi/meet-events.yaml
buf lint contracts/proto/meet.proto
cargo run -p oya-dev-cli -- gate validate contract-stability --microservice meet
```

## Test Plan

- OpenAPI: spectral lint clean; OAS-version-pin; required X-Scope-OrgID + X-Context-Kind headers present.
- AsyncAPI: schema valid; all events have payload schemas; both WebSocket and AMQP channels declared.
- Proto: buf lint clean; backward-compatibility check vs prior release pointer.

## Next IP

[`IP-014-cedar-policies-and-data-residency.md`](IP-014-cedar-policies-and-data-residency.md)

## References

- OpenAPI 3.2 spec.
- AsyncAPI 3.1 spec.
- Protocol Buffers v3 + gRPC.
- spectral rulesets; buf lint.
