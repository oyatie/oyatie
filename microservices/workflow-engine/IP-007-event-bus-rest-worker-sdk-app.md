---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-007-event-bus-rest-worker-sdk-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: oya-workflow-engine-event-bus-{rest,worker,sdk,app}

## Intent

Complete the event-bus BC: REST surface for tenants + Studio + workload µservices to publish/subscribe; outbox-relay worker; SDK consumed by every µservice; app composition root.

## ChangeSet boundary

4 new crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-workflow-engine-event-bus-rest/{Cargo.toml,src/{lib,routes,middleware}.rs}` | create | HTTP routes per `contracts/openapi/workflow-engine.yaml`; OIDC + Cedar middleware |
| `src/crates/oya-workflow-engine-event-bus-worker/{Cargo.toml,src/{lib,outbox_relay,delivery}.rs}` | create | Long-lived outbox relay; HA via Redis lease |
| `src/crates/oya-workflow-engine-event-bus-sdk/{Cargo.toml,src/{lib,client,event_types}.rs}` | create | Tenant SDK; type registry; idempotency-key helpers |
| `src/crates/oya-workflow-engine-event-bus-app/{Cargo.toml,src/main.rs}` | create | Composition root binary |
| `microservices/workflow-engine/catalog/oya-workflow-engine-event-bus-*.yaml` | create | 4 catalog rows |

## Acceptance Gates

```bash
cargo nextest run -p oya-workflow-engine-event-bus-rest --all-features
cargo nextest run -p oya-workflow-engine-event-bus-worker --all-features
cargo nextest run -p oya-workflow-engine-event-bus-sdk --all-features
cargo nextest run -p oya-workflow-engine-event-bus-app --all-features
cargo run -p oya-dev-cli -- gate validate openapi-conformance --crate oya-workflow-engine-event-bus-rest
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_rest_route_publish_tenant_stamping` | client cannot override tenant_id |
| `test_rest_route_subscribe_tenant_isolation` | subscription bound to authenticated tenant |
| `test_outbox_relay_crash_recovery` | HA failover resumes from last offset |
| `test_outbox_relay_at_least_once_delivery` | event delivered to subscriber ≥ 1 |
| `test_sdk_idempotency_key_helper` | helper generates ULID-based key |
| `test_sdk_publish_subscribe_roundtrip` | E2E against rest crate |

## Next IP

[`IP-008-spec-store-usecase-api-adapter-rest-sdk-app.md`](IP-008-spec-store-usecase-api-adapter-rest-sdk-app.md)

## References

- PRD FR-03, FR-04, FR-13
- `contracts/openapi/workflow-engine.yaml`
- `contracts/asyncapi/workflow-events.yaml`
- `sdk-plan.md`
