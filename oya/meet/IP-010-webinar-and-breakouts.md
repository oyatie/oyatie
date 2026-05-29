---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-010-webinar-and-breakouts
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet
acceptance_lanes: [cargo-nextest, webinar-fanout-load-test]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: webinar mode (registration + practice + Q&A) + breakout rooms

## Intent

Author the webinar BC + breakout-rooms feature inside meeting-instance BC. Webinar mode introduces registration (pre-meeting attendee pre-registration with custom fields), practice session (host + co-host private "green room" before going live), Q&A moderation (queued questions; moderator filters before showing to host), attendee report (post-event analytics). Breakout rooms: host splits attendees into N sub-rooms then re-merges.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-meet-webinar-{kernel,domain,usecase}/src/...` | create |
| `src/crates/oya-meet-webinar-adapter-postgres/src/...` | create — registration table |
| `src/crates/oya-meet-webinar-rest/src/handlers.rs` | create |
| `src/crates/oya-meet-webinar-worker/src/registration_consumer.rs` | create |
| `src/crates/oya-meet-meeting-instance-usecase/src/breakout.rs` | create — breakout-rooms usecase |
| `tests/webinar_lifecycle_e2e.rs` | create |
| `tests/breakout_rooms_e2e.rs` | create |

## Code Shape

```rust
// webinar/usecase
pub struct OpenRegistration;
impl OpenRegistration {
    pub async fn execute(&self, ctx: &Ctx, host: &Principal, webinar_id: WebinarId, opts: RegistrationOpts) -> Result<RegistrationHandle> {
        ctx.cedar.require(Action::OpenRegistration, host, webinar_id).await?;
        let reg = ctx.repo.create_registration_form(webinar_id, opts).await?;
        ctx.audit.seal(WebinarRegistrationOpened { webinar_id, opened_by: host.id() }).await?;
        Ok(reg)
    }
}

// meeting-instance/usecase: breakout
pub struct CreateBreakoutRooms;
impl CreateBreakoutRooms {
    pub async fn execute(&self, ctx: &Ctx, host: &Principal, instance_id: &InstanceId, config: BreakoutConfig) -> Result<Vec<BreakoutRoom>> {
        ctx.cedar.require(Action::CreateBreakoutRooms, host, instance_id).await?;
        let rooms = config.assignments.iter().map(|a| {
            // Issue per-breakout LiveKit subroom token
            ctx.sfu.create_breakout_subroom(instance_id, a)
        }).collect::<Result<Vec<_>>>()?;
        ctx.audit.seal(BreakoutRoomsCreated { instance_id, count: rooms.len() }).await?;
        Ok(rooms)
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-meet-webinar-rest
cargo nextest run --test webinar_lifecycle_e2e
cargo nextest run --test breakout_rooms_e2e
# Load-test: 10k attendees registration + fan-out p99 ≤ 5s
```

## Test Plan

- Webinar registration: pre-registration form accepts custom fields; rate-limited.
- Practice session: host + co-host can rehearse before going live; attendees in lobby.
- Q&A queue: moderator approves question → visible to host + audience.
- Attendee report: post-event analytics include join/leave timing + engagement metrics.
- Breakout rooms: 1 main room + 5 breakouts; attendees auto-route + re-merge cleanly.
- 10k attendee fan-out: WHIP/HLS mesh kicks in at ≥ 1000; broadcast attendees receive within 5s.

## Next IP

[`IP-011-live-stream-egress.md`](IP-011-live-stream-egress.md)

## References

- ADR-MEET-0005 (large-audience + webinar architecture).
- Zoom Webinar reference.
- LiveKit Egress + Composite.
- IETF MoQ (Media-over-QUIC).
