---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-004-professional-graph-and-connection-request-bcs
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location, oya-governance-statelessness, oya-governance-professional-context-isolation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: professional-graph + connection-request BCs end-to-end

## Intent

Author both BCs together (tightly coupled):

- `professional-graph`: 1st/2nd/3rd-degree connection edges (directed-bidirectional-on-acceptance); follow edges; block / restrict / disconnect; BFS-based degree-of-separation up to depth 3 with Valkey cache.
- `connection-request`: lifecycle (open → accepted | rejected | ignored | withdrawn); per-user-per-week rate limit; spam classifier signal.

Land the `synchronous_commit = remote_write` consistency posture per ADR-NET-0001.

## ChangeSet boundary

Both BCs across all layers; shared `network_connection_edges` Postgres table; Valkey cache for degree-of-separation.

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait ProfessionalGraphRepository: Send + Sync {
    async fn upsert_connection_edge(&self, edge: ConnectionEdge) -> Result<(), GraphError>;
    async fn remove_connection_edge(&self, tenant_id: &TenantId, a: &UserRef, b: &UserRef) -> Result<(), GraphError>;
    async fn list_first_degree(&self, tenant_id: &TenantId, user: &UserRef) -> Result<Vec<UserRef>, GraphError>;
    async fn compute_degree(&self, tenant_id: &TenantId, from: &UserRef, to: &UserRef) -> Result<DegreeOfSeparation, GraphError>;
}

#[async_trait]
pub trait ConnectionRequestRepository: Send + Sync {
    async fn create(&self, req: ConnectionRequestNew) -> Result<ConnectionRequest, ConnReqError>;
    async fn respond(&self, tenant_id: &TenantId, req_id: &ConnectionRequestId, verdict: Verdict) -> Result<ConnectionRequest, ConnReqError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-professional-graph-kernel
cargo nextest run -p oya-network-professional-graph-adapter-postgres
cargo nextest run -p oya-network-connection-request-kernel
cargo run -p oya-dev-cli -- gate validate statelessness --microservice network
cargo run -p oya-dev-cli -- gate validate professional-context-isolation --microservice network
```

## Test Plan

- Degree-of-separation BFS cap test: depth-3 limit; out-of-range → `DegreeOfSeparation::Out`.
- Per-tenant connection-edge advisory-lock concurrency test (preview of ADR-NET-0005 endorsement-chain ordering pattern).
- Connection-request rate-limit test: 500/week/account cap.
- Cross-tenant edge insertion forbidden at PG + Cedar layers.

## Halt Conditions

- Cross-tenant edge inserts compile or pass at runtime.

## Next IP

[`IP-005-post-composition-bc.md`](IP-005-post-composition-bc.md)

## References

- ADR-NET-0001 (storage; advisory-lock pattern).
- `policy/professional-context-isolation.md`.
- LinkedIn FollowGraph engineering blog (reference pattern).
