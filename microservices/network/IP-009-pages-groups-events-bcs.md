---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-009-pages-groups-events-bcs
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network + axis-mail + axis-calendar
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location]
---

# IP-009: pages + groups + events-bridge BCs end-to-end

## Intent

Land three BCs together:

- `pages`: Company / brand Pages; multi-admin; newsletter-bridge to mail µservice; analytics; verified Page badge.
- `groups`: Private + open groups; per-group feed; moderation; admin / moderator roles; join-request flow.
- `events-bridge`: Professional events with calendar µservice bridge; RSVP; capacity; recurring; iCal export.

## Code Shape

```rust
// pages kernel/src/ports.rs
#[async_trait]
pub trait PageRepository: Send + Sync {
    async fn create(&self, page: PageNew) -> Result<Page, PageError>;
    async fn add_admin(&self, page_id: &PageId, admin: &UserRef) -> Result<(), PageError>;
    async fn publish_newsletter(&self, page_id: &PageId, body: NewsletterBody) -> Result<NewsletterId, PageError>;
}

// groups kernel/src/ports.rs
#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn create(&self, group: GroupNew) -> Result<Group, GroupError>;
    async fn join_request(&self, group_id: &GroupId, applicant: &UserRef) -> Result<JoinRequest, GroupError>;
    async fn approve(&self, jr_id: &JoinRequestId, approver: &GroupAdminRef) -> Result<GroupMembership, GroupError>;
}

// events-bridge kernel/src/ports.rs
#[async_trait]
pub trait EventsBridge: Send + Sync {
    async fn create_event(&self, ev: NetworkEventNew) -> Result<NetworkEvent, EventError>;
    async fn rsvp(&self, event_id: &EventId, user: &UserRef, verdict: RsvpVerdict) -> Result<RSVP, EventError>;
    async fn emit_to_calendar(&self, event: &NetworkEvent) -> Result<CalendarBridgeReceipt, EventError>;
    async fn emit_ical(&self, event: &NetworkEvent) -> Result<Vec<u8>, EventError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-pages-kernel
cargo nextest run -p oya-network-groups-kernel
cargo nextest run -p oya-network-events-bridge-kernel
```

## Test Plan

- Page newsletter emission: `NewsletterSendRequested` event reaches mail µservice; mail confirms receipt.
- Group join-request approval: group-admin only; per-tenant Cedar.
- Event RSVP: capacity-bound; waitlist when full.
- iCal export: RFC 5545 compliance; calendar µservice receives bridge event.

## Halt Conditions

- Cross-µservice bridge degraded (mail / calendar Sev-2) — queue + replay per `runbooks/inmail-fanout-degraded.md` pattern.

## Next IP

[`IP-010-inmail-bridge-bc.md`](IP-010-inmail-bridge-bc.md)

## References

- ADR-0131 (bridge crate convention).
- RFC 5545 (iCalendar).
- mail + calendar µservice docs.
