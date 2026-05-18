---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-007-screen-share-and-tracks
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet
acceptance_lanes: [cargo-nextest, screen-share-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: audio + video + screen-share track lifecycle

## Intent

Author the audio + video + screen-share BCs as thin adapters over LiveKit track lifecycle. Audio: mute/echo-cancel/noise-suppression (delegated to LiveKit). Video: HD/4K/spotlight/virtual-background/blur. Screen-share: track publish + presenter-control (only host or designated presenter can publish) + remote-control grant (Cedar `Action::grant_remote_control`).

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-meet-audio-{kernel,domain,usecase,adapter-livekit,worker,sdk}/src/...` | create |
| `src/crates/oya-meet-video-{kernel,domain,usecase,adapter-livekit,worker,sdk}/src/...` | create |
| `src/crates/oya-meet-screen-share-{kernel,domain,usecase,adapter-livekit,worker,sdk}/src/...` | create |
| `tests/screen_share_e2e.rs` | create |

## Code Shape

```rust
// screen-share/usecase
pub struct StartScreenShare;
impl StartScreenShare {
    pub async fn execute(&self, ctx: &Ctx, principal: &Principal, instance_id: &InstanceId) -> Result<TrackHandle> {
        // Presenter-control: only one screen-share at a time per room
        if let Some(existing) = ctx.sfu.get_active_screen_share(instance_id).await? {
            if existing.publisher != principal.id() {
                return Err(Error::ScreenSharePresenterControl(existing.publisher));
            }
        }
        // Cedar gate
        ctx.cedar.require(Action::StartScreenShare, principal, instance_id).await?;
        // Issue track-publish capability
        let handle = ctx.sfu.publish_screen_share(instance_id, principal).await?;
        ctx.audit.seal(ScreenShareStarted { instance_id, publisher: principal.id() }).await?;
        Ok(handle)
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-meet-screen-share-adapter-livekit
cargo nextest run --test screen_share_e2e
```

## Test Plan

- Screen-share start p95 ≤ 800ms.
- Presenter-control: second user attempts to start → refused with `Error::ScreenSharePresenterControl`.
- Remote-control grant + revoke: integration with Cedar action.
- Video simulcast: publisher emits low/mid/high streams; SFU selects per subscriber.

## Next IP

[`IP-008-recording-pipeline.md`](IP-008-recording-pipeline.md)

## References

- ADR-MEET-0001.
- LiveKit screen-share docs.
- W3C Screen Capture API.
