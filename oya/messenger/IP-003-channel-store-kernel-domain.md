---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-003-channel-store-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location, oya-governance-layer-correctness, oya-governance-dual-context-isolation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: channel-store kernel + domain

## Intent

Author kernel port traits (`ChannelRepository`, `CedarChannelPolicy`,
`AuditChainClient`) and domain entities (`Channel`, `DirectConversation`,
`ChannelMember`, `RetentionPolicy`, `Hold`). Land the `ContextKind` sealed enum
per `policy/dual-context-isolation.md` DCI-01.

## ChangeSet boundary

`-kernel` + `-domain` crates only; no I/O. No `-adapter-*` yet (IP-004).

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-channel-store-kernel/src/{ports,entities,context_kind,errors}.rs` | create |
| `src/crates/oya-messenger-channel-store-domain/src/{channel,direct_conversation,member,retention,hold}.rs` | create |
| `tests/dual_context_invariant.rs` | create — UI tests asserting cross-context impl-coverage rejected |

## Code Shape

```rust
// kernel/src/context_kind.rs
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextKind { Personal, Professional }

// kernel/src/ports.rs
#[async_trait]
pub trait ChannelRepository: Send + Sync {
    async fn create(&self, channel: Channel) -> Result<Channel, ChannelError>;
    async fn get(&self, tenant_id: &TenantId, channel_id: &ChannelId)
        -> Result<Option<Channel>, ChannelError>;
    async fn list_for_member(&self, tenant_id: &TenantId, user_ref: &UserRef,
        context: ContextKind)
        -> Result<Vec<Channel>, ChannelError>;
    async fn archive(&self, tenant_id: &TenantId, channel_id: &ChannelId)
        -> Result<(), ChannelError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-channel-store-kernel
cargo nextest run -p oya-messenger-channel-store-domain
buck2 build //:quality-lane-registry-authority-check # lane=port-location --microservice messenger
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice messenger
buck2 build //:quality-lane-registry-authority-check # lane=dual-context-isolation --microservice messenger
```

## Test Plan

- Unit tests on each entity invariant (e.g., Channel always Professional;
  DirectConversation always Personal).
- UI tests: attempting to mix context types fails to compile.

## Halt Conditions

- Any cross-context coercion compiles — bug; fix the type system.
- Any port trait declares I/O dependency — kernel-purity violation.

## Next IP

[`IP-004-channel-store-adapter-postgres.md`](IP-004-channel-store-adapter-postgres.md)

## References

- `policy/dual-context-isolation.md` DCI-01..DCI-07.
- Bominal ADR-0208; parallel ADR-0238.
