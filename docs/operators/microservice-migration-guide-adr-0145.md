# Microservice migration guide — ADR-0145 adoption (6-step)

**Status**: Active 2026-05-18
**Source ADR**: [ADR-0145 — inter-microservice communication reform](../decisions/ADR-0145-inter-microservice-communication-reform.md)
**Audience**: per-µservice axis owners migrating to the hyperscaler shape.

This guide codifies the 6-step adoption every µservice follows to migrate from the Workflow+Ontology universal-mediator pattern (pre-ADR-0145) to the hyperscaler-shape pattern (post-ADR-0145). Pair with [ADR-0145-runtime-impact-changelog.md](ADR-0145-runtime-impact-changelog.md) for operator-side impact.

## Pre-flight

Verify the µservice has:

- A `manifest.json` validated by `specs/microservices/manifest-schema.json`.
- A `policy/tenant-scope.cedar` fragment.
- An `iac/helm/<ms>/templates/networkpolicy.yaml`.
- A `scorecards/overrides.json`.

If any of these are missing, see `docs/templates/microservice-bootstrap-checklist.md` first.

## Step 1 — Integrate audit-chain client (Invariant 1)

Add to the µservice's `Cargo.toml` (one of the µservice's app crates, typically `crates/oya-<ms>-app/Cargo.toml`):

```toml
[dependencies]
shared-audit-chain-client-kernel = { path = "../shared-audit-chain-client-kernel" }
```

At every state-changing capability site, emit a seal:

```rust
use shared_audit_chain_client_kernel::{
    AuditChainClient, CallingMicroservice, CalledMicroservice,
    SealEmission, SealEventKind,
};

client.emit_seal(&SealEmission {
    from: CallingMicroservice("<your-ms>".into()),
    to: CalledMicroservice("<sibling-ms>".into()),
    capability_id: "<capability-id>".into(),
    event_kind: SealEventKind::StateChange,
    trace_id,
    payload_digest_hex,
})?;
```

During the skeleton phase, use `NoopAuditChainClient` as the integration target. Production impl lands per `registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-audit-client-impl`.

## Step 2 — Integrate tracing client (Invariant 2)

Add to `Cargo.toml`:

```toml
[dependencies]
shared-tracing-client-kernel = { path = "../shared-tracing-client-kernel" }
```

On every outbound gRPC call, inject the W3C traceparent:

```rust
use shared_tracing_client_kernel::{TracingClient, NoopTracingClient};

let mut headers = std::collections::BTreeMap::new();
client.inject(&mut headers)?;
// attach `headers` to the outbound request metadata
```

On every inbound gRPC call, extract:

```rust
let inbound = client.extract(&headers)?;
// inbound.traceparent → propagate to internal span context
```

## Step 3 — Relax NetworkPolicy egress (where appropriate)

Edit `iac/helm/<ms>/templates/networkpolicy.yaml`:

- Add direct egress entries for sibling µservices the µservice now calls without workflow-engine mediation.
- For state-changing calls that retain saga/durable semantics, keep the workflow-engine egress (per the rubric in `docs/standards/workflow-vs-direct-grpc-rubric.md`).
- Add the Cilium mesh policy entries (CiliumNetworkPolicy identity-based L4 rules + L7 HTTP/gRPC rules where needed) per ADR-0148. For Tier-2 Istio Ambient namespaces (initially `workflow-engine`, `foundry-orchestrator`), additionally add the Ambient waypoint AuthorizationPolicy entries.

## Step 4 — Author Cedar policy fragments for sibling-µservice principals

Edit `policy/tenant-scope.cedar`:

- For every sibling µservice that calls into THIS µservice, declare a `permit (principal in SiblingMicroservice::"<sibling-ms>", ...)` rule pinned to the specific Action token.
- Use the canonical sibling-call Cedar fragments from `microservices/governance/policy/canonical-sibling-call-fragments.cedar` (see governance µservice for the catalog).

## Step 5 — Declare ontology projections (Invariant 3)

If this µservice owns canonical entities (Person, Task, Document, Recording, etc.), add to `manifest.json`:

```json
"ontology_projections": [
  {
    "entity_name": "Person",
    "projection_target_table": "ontology_persons",
    "projection_kind": "idempotent-rewrite",
    "lag_budget_seconds": 60
  }
]
```

The schema is enforced by `specs/microservices/manifest-schema.json`. The gate `oya gate validate ontology-projection-coverage` flags µservices that own entities but ship an empty projections list.

## Step 6 — Wire the per-µservice metric kernel

Add to `Cargo.toml`:

```toml
[dependencies]
shared-hyperscaler-metrics-kernel = { path = "../shared-hyperscaler-metrics-kernel" }
shared-hyperscaler-metrics-adapter-prometheus = { path = "../shared-hyperscaler-metrics-adapter-prometheus" }
```

Export the canonical hyperscaler-bar metrics (request rate, error rate, latency p50/p99, in-flight requests) per inter-µservice call surface. These ground the SLO claims declared in `slos/*.openslo.yaml`.

## Verification

After all 6 steps land:

```bash
# Build the µservice
cargo build -p oya-<ms>-app

# Run the µservice's tests
cargo nextest run -p oya-<ms>-app

# Validate manifest
cargo run -p dev-cli -- gate validate ontology-projection-coverage

# Validate audit-chain coverage
cargo run -p dev-cli -- gate validate audit-chain-seal-coverage

# Validate trace propagation
cargo run -p dev-cli -- gate validate otel-trace-propagation
```

All three gates run in DEFERRED (advisory) mode during the skeleton phase; they will surface findings without failing the gate run-all.

## Rollback (per µservice)

Each µservice migration is a single PR. To revert:

```bash
git revert <pr-merge-commit>
```

The skeleton clients (`Noop*Client`) and the advisory-mode gates ensure no state-change one-way during the migration.

## References

- ADR-0145 — inter-microservice communication reform.
- ADR-0148 — Cilium Service Mesh (primary) + Istio Ambient waypoint (Tier-2 opt-in).
- docs/operators/ADR-0145-runtime-impact-changelog.md — operator-side changelog.
- docs/standards/workflow-vs-direct-grpc-rubric.md — when to use which path.
- registry/placeholder-debt/adr-follow-ups.yaml — skeleton-impl tracking.
