---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-012-replay-debugger-backend-rest-sdk-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: oya-workflow-engine-replay-debugger-backend-{rest,sdk,app}

## Intent

Complete replay-debugger-backend BC: rest surface (consumed by Studio); SDK (tenant programmatic replay); app composition root.

## ChangeSet boundary

3 new crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-workflow-engine-replay-debugger-backend-rest/{...}` | create | HTTP routes per `contracts/openapi/workflow-engine.yaml`; gRPC streaming for step snapshots |
| `src/crates/oya-workflow-engine-replay-debugger-backend-sdk/{...}` | create | Tenant SDK |
| `src/crates/oya-workflow-engine-replay-debugger-backend-app/{...}` | create | Composition root |
| `microservices/workflow-engine/catalog/oya-workflow-engine-replay-debugger-backend-*.yaml` | create | 3 catalog rows |

## Test Plan

| Test | Verifies |
|---|---|
| `test_rest_replay_endpoint_happy` | replay session created, streamed |
| `test_streaming_step_snapshots` | grpc stream delivers snapshots in order |
| `test_auditor_scope_read_only` | auditor token cannot trigger side-effecting replay |

## Next IP

[`IP-013-observability-slo-manifests.md`](IP-013-observability-slo-manifests.md)

## References

- PRD AC-02
- `contracts/openapi/workflow-engine.yaml`
- `policy/auditor-scope.cedar`
