---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-007-supervision-event-bus
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1]
depends_on: [IP-002, IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: supervision-event-bus BC (7 crates)

## Intent

AMQP + Valkey Streams (Redis wire-compat) substrate; per-event Ed25519 signature; subscriber registration. 7 crates: kernel, usecase, api, adapter, worker, sdk, app.

## Concrete File Targets

Crates at `microservices/intelligence/src/crates/oya-foundry-supervisor-supervision-event-bus-{layer}/`.

## Key code

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait SupervisionEventPublisher: Send + Sync + Sealed {
    async fn publish(&self, event: &SupervisionEvent) -> Result<EventId, KernelError>;
}

#[async_trait]
pub trait SupervisionEventSubscriber: Send + Sync + Sealed {
    async fn subscribe(&self, topic: &str, group: &str) -> Result<Box<dyn Stream<Item = Result<SupervisionEvent, KernelError>>>, KernelError>;
}
```

```rust
// adapter/src/redis_streams.rs
// Implements both Publisher (XADD with Ed25519-signed payload) and Subscriber
// (XREADGROUP with at-least-once delivery).
```

## Acceptance Gates

Standard per-crate gates. Plus:

```bash
# End-to-end: publish synthetic event; verify foundry-evidence sealed; verify audit-chain Merkle.
cargo nextest run -p oya-foundry-supervisor-supervision-event-bus-worker --test e2e_publish_and_seal
```

## Halt Conditions

- Event missing Ed25519 signature.
- At-least-once delivery semantics violated.

## Next IP

[`IP-008-kill-switch-engage-state.md`](IP-008-kill-switch-engage-state.md)

## References

- PRD FR-06; `contracts/asyncapi/foundry-supervisor-events.yaml`.
- ADR-0028 (audit-chain).
- Valkey Streams (Redis wire-compat) — `redis.io/docs/data-types/streams/`.

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `supervisor`-bounded-context slice for `IP-007: supervision-event-bus BC (7 crates)`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: tenant-visible fleet control with kill-switch and capability deployment evidence. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/intelligence/capabilities/supervisor-deploy-capability.yaml`, `microservices/intelligence/capabilities/supervisor-engage-kill-switch.yaml`, `microservices/intelligence/capabilities/supervisor-query-fleet-state.yaml`, `microservices/intelligence/contracts/openapi/supervisor-foundry-supervisor.yaml`, and the policy set `microservices/intelligence/policy/supervisor-tenant-scope.cedar`, `microservices/intelligence/policy/supervisor-supervisor-isolation.md`, `microservices/intelligence/policy/supervisor-ci-scope.cedar`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `supervisor` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/intelligence/manifest.json`; the primary implementation anchor is `crates/oya-intelligence-supervisor-kernel/src/lib.rs` plus the matching catalog records under `microservices/intelligence/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/intelligence/contracts/openapi/supervisor-foundry-supervisor.yaml`, `microservices/intelligence/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`, and `microservices/intelligence/contracts/proto/supervisor-foundry-supervisor.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/intelligence/policy/supervisor-tenant-scope.cedar`, `microservices/intelligence/policy/supervisor-supervisor-isolation.md`, `microservices/intelligence/policy/supervisor-ci-scope.cedar`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/intelligence/slos/supervisor-command-propagation.openslo.yaml`, `microservices/intelligence/slos/supervisor-fleet-state-freshness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/intelligence/capabilities/supervisor-deploy-capability.yaml`, `microservices/intelligence/capabilities/supervisor-engage-kill-switch.yaml`, `microservices/intelligence/capabilities/supervisor-query-fleet-state.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/intelligence/PRD.md` and the `supervisor` row in `microservices/intelligence/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/intelligence/catalog/`, `microservices/intelligence/contracts/`, `microservices/intelligence/policy/`, or `microservices/intelligence/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-intelligence-supervisor-kernel/src/lib.rs`.
- Contract parity for `microservices/intelligence/contracts/openapi/supervisor-foundry-supervisor.yaml` and `microservices/intelligence/contracts/proto/supervisor-foundry-supervisor.proto` when DTOs or handlers change.
- Policy resolution against `microservices/intelligence/policy/supervisor-tenant-scope.cedar`, `microservices/intelligence/policy/supervisor-supervisor-isolation.md`, `microservices/intelligence/policy/supervisor-ci-scope.cedar`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/intelligence/slos/supervisor-command-propagation.openslo.yaml`, `microservices/intelligence/slos/supervisor-fleet-state-freshness.openslo.yaml`; no acceptance by line count alone.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/intelligence/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/intelligence/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| Palantir AIP Operator and Azure AI Foundry deployment controls | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
