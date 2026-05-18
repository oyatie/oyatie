---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-003-meeting-room-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet
acceptance_lanes: [cargo-nextest, oya-governance-port-location, oya-governance-data-class]
---

# IP-003: meeting-room kernel + domain

## Intent

Author the meeting-room BC's kernel + domain layers per ADR-0105. Kernel declares port traits (`MeetingRoomRepository`, `CedarMeetingPolicy`, `AuditChainClient`) and entities (`MeetingRoom`, `LobbyPolicy`, `WaitingRoomPolicy`, `RetentionBinding`). Domain houses pure rules (room-name validation per BNF, lobby-policy state machine, retention-binding resolution).

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-meet-meeting-room-kernel/src/lib.rs` | create — trait + entity declarations |
| `src/crates/oya-meet-meeting-room-kernel/src/entities/{meeting_room,lobby_policy,waiting_room_policy,retention_binding}.rs` | create |
| `src/crates/oya-meet-meeting-room-kernel/src/ports/{repository,cedar_policy,audit_chain}.rs` | create |
| `src/crates/oya-meet-meeting-room-domain/src/lib.rs` | create — pure rules |
| `src/crates/oya-meet-meeting-room-domain/src/rules/{name_validation,lobby_state_machine,retention_resolver}.rs` | create |

## Code Shape

```rust
// kernel/src/entities/meeting_room.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
#[data_class = "BEHAVIORAL_TENANT_PRODUCT"]
pub struct MeetingRoom {
    pub room_id: RoomId,            // ULID
    pub tenant_id: TenantId,        // ADR-0140 sharding key
    pub name: RoomName,             // BNF-validated
    pub topic: Option<RoomTopic>,
    pub lobby_policy: LobbyPolicy,
    pub waiting_room_policy: WaitingRoomPolicy,
    pub retention_binding: RetentionBinding,
    pub created_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait MeetingRoomRepository: Send + Sync {
    async fn create(&self, room: MeetingRoom) -> Result<MeetingRoom>;
    async fn get(&self, room_id: &RoomId, tenant_id: &TenantId) -> Result<Option<MeetingRoom>>;
    async fn list(&self, tenant_id: &TenantId, cursor: Option<&str>, limit: u32) -> Result<(Vec<MeetingRoom>, Option<String>)>;
    async fn archive(&self, room_id: &RoomId, tenant_id: &TenantId) -> Result<()>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-meet-meeting-room-kernel
cargo nextest run -p oya-meet-meeting-room-domain
cargo run -p oya-dev-cli -- gate validate port-location --microservice meet
cargo run -p oya-dev-cli -- gate validate data-class --microservice meet
```

## Test Plan

- Trait shape: doctest on every port trait.
- Entity invariants: prop-test on `RoomName::parse()` (BNF-conformant).
- Lobby state machine: exhaustive transition table; rejected transitions return typed error.

## Halt Conditions

- Port trait carries any I/O — refuse; move to adapter layer.
- Unannotated data-class on a field — refuse.

## Next IP

[`IP-004-meeting-room-adapter-postgres.md`](IP-004-meeting-room-adapter-postgres.md)

## References

- ADR-0105; ADR-0106; ADR-0131; ADR-MEET-0001.
