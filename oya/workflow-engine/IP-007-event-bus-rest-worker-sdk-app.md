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
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, openapi-conformance, asyncapi-conformance]
---

# IP-007: tenant-scoped event bus REST, relay worker, SDK, and app root

## §A Problem

The event bus is the workflow-engine boundary that turns durable execution into a cross-µservice substrate. `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml` already defines tenant-hashed topics and lifecycle events, while `contracts/openapi/workflow-engine.yaml` exposes `/events`, `/subscriptions/{sub_id}/replay`, and related REST operations. The stamped version of this IP only named four crates; it did not specify how the REST surface, outbox relay, tenant SDK, and composition root preserve idempotency, tenant stamping, and replay-safe offsets.

The real gap is not "create an event bus". It is: a workflow event published by one tenant must be server-stamped, persisted once, delivered at least once, replayable by offset, denied across tenant boundaries, and observable enough to debug a subscriber that replays old `StepCompleted` events after an outage.

## §B Approach

Implement the remaining event-bus layers listed in `microservices/workflow-engine/manifest.json`: `oya-workflow-engine-event-bus-rest`, `worker`, `sdk`, and `app`. REST accepts tenant-authenticated publish/subscribe/replay calls, but never trusts a caller-supplied `tenant_id`. The worker drains the Postgres outbox to the per-pack NATS JetStream channel named by AsyncAPI (`oya.workflow-engine.{tenant_hash}.{event_type}`), using Valkey leases only for relay ownership and never as event authority. The SDK exposes typed event constructors so sibling µservices do not hand-roll message envelopes.

Non-goals are explicit: this IP does not introduce new lifecycle event names, does not change the AsyncAPI topic namespace, and does not make NATS/Valkey authoritative for workflow history. Those concerns belong to the event-bus kernel/domain or Layer-A infrastructure IPs.

## §C Deliverables

| Artifact | Action | Substance requirement |
|---|---|---|
| `microservices/workflow-engine/src/crates/oya-workflow-engine-event-bus-rest/src/routes.rs` | create | handlers for publish, subscription create/delete, event replay, and stream bootstrap mapped to OpenAPI operation IDs |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-event-bus-rest/src/middleware.rs` | create | OIDC extraction plus `policy/tenant-scope.cedar` checks for publish/subscribe/replay |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-event-bus-worker/src/outbox_relay.rs` | create | leases a shard, reads durable outbox rows, publishes to AsyncAPI channels, records delivery offsets |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-event-bus-worker/src/delivery.rs` | create | retry ladder, poison-message quarantine, and subscriber replay cursor handling |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-event-bus-sdk/src/event_types.rs` | create | typed constructors for `WorkflowStarted`, `StepStarted`, `StepCompleted`, `StepFailed`, `StepRetried`, terminal events |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-event-bus-sdk/src/client.rs` | create | publish/subscribe/replay client with idempotency-key helper |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-event-bus-app/src/main.rs` | create | composition root wiring REST config, relay config, Cedar bundle, OpenBao refs, and metrics exporters |
| `microservices/workflow-engine/catalog/oya-workflow-engine-event-bus-{rest,worker,sdk,app}.yaml` | update/create | catalog rows for the four crates |

## §D Implementation

1. Map OpenAPI `/events` publish and `/subscriptions/{sub_id}/replay` operations into REST route functions with request structs generated from the contract fields.
2. Build middleware that resolves authenticated `tenant_id`, hashes it for topic routing, and rejects mismatched resource tenant data using `policy/tenant-scope.cedar`.
3. Model outbox relay state as durable Postgres offset plus ephemeral Valkey lease; relay restart must resume from the last acknowledged outbox id.
4. Publish only AsyncAPI-declared lifecycle event types; unknown event types are refused before persistence to avoid subscriber-specific topic drift.
5. Add SDK helpers for idempotency keys, correlation ids, and strongly typed lifecycle payloads so `identity`, `payments`, `messenger`, and Foundry callers share one event envelope.
6. Emit metrics named from service vocabulary: publish latency, relay lag, poison-message count, replay cursor lag, and unauthorized subscription attempts.
7. Wire the app crate with config validation for per-pack endpoint, NATS subject prefix, Postgres outbox DSN, Valkey Sentinel endpoints, Cedar bundle path, and OpenBao secret references.

## §E Acceptance

- `cargo nextest run -p oya-workflow-engine-event-bus-rest --all-features`
- `cargo nextest run -p oya-workflow-engine-event-bus-worker --all-features`
- `cargo nextest run -p oya-workflow-engine-event-bus-sdk --all-features`
- `cargo nextest run -p oya-workflow-engine-event-bus-app --all-features`
- `buck2 build //:quality-lane-registry-authority-check # lane=openapi-conformance --crate oya-workflow-engine-event-bus-rest`
- `buck2 build //:quality-lane-registry-authority-check # lane=asyncapi-conformance --microservice workflow-engine`
- Required tests: `publish_server_stamps_tenant_id`, `subscribe_cross_tenant_denied`, `outbox_relay_resumes_after_crash`, `replay_from_offset_is_idempotent`, `sdk_rejects_unknown_event_type`, and `poison_message_quarantines_without_blocking_tenant_topic`.

## §F Evidence

- `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml` defines topic shape, lifecycle messages, idempotency, and tenant-hash isolation.
- `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml` defines REST publish, subscribe, and replay surfaces.
- `microservices/workflow-engine/policy/tenant-scope.cedar` permits only same-tenant event publish/subscribe/replay.
- `microservices/workflow-engine/runbooks/event-bus-replay.md` and `runbooks/valkey-failover.md` provide operational recovery paths for replay and lease-state failure.
- `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml` and `workflow-step-execute-latency.openslo.yaml` are the monitoring anchors for worker health.

## §G Counterparts

| Counterpart | Relevant behavior | This IP closes |
|---|---|---|
| Temporal / Cadence | history service and matching service separate durable history from worker polling | durable outbox plus relay worker separates event authority from transient delivery |
| AWS Step Functions | execution events are queryable and replayable through history APIs | `/subscriptions/{sub_id}/replay` and offset cursors make workflow events replayable |
| n8n | webhooks and node events are broad but not tenant-hashed durable substrate events | tenant-hashed topics and Cedar-denied subscriptions close multi-tenant leakage |
| GitHub Actions | workflow events drive downstream checks with stable ids | SDK idempotency and correlation ids prevent duplicate downstream effects |

## Next IP

[`IP-008-spec-store-usecase-api-adapter-rest-sdk-app.md`](IP-008-spec-store-usecase-api-adapter-rest-sdk-app.md)

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-007-event-bus-rest-worker-sdk-app.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.
