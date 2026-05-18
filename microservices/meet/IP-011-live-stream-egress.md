---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-011-live-stream-egress
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet + ops-security
acceptance_lanes: [cargo-nextest, rtmp-smoke, oya-governance-egress-allow-list]
---

# IP-011: live-stream egress (RTMP to YouTube/Twitch/Vimeo + WHIP fallback)

## Intent

Author the live-stream-egress BC: SRS 6.0 RTMP server (sidecar) ingests LiveKit composite feed; outbound RTMP to external streaming platforms (YouTube Live, Twitch, Vimeo Live, custom RTMP endpoints). Per-tenant egress allow-list enforced at NetworkPolicy + DNS allow-list + Cedar `Action::StartLiveStreamEgress`. WHIP (RFC draft) fallback for platforms that prefer WebRTC ingest. Egress key held only at egress worker via OpenBao.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-meet-live-stream-egress-{kernel,domain,usecase}/src/...` | create |
| `src/crates/oya-meet-live-stream-egress-adapter-srs/src/client.rs` | create — SRS API client; RTMP outbound publish |
| `src/crates/oya-meet-live-stream-egress-adapter-ffmpeg/src/transcode.rs` | create — re-encode if platform requires specific bitrate/codec |
| `src/crates/oya-meet-live-stream-egress-worker/src/...` | create |
| `iac/helm/meet/templates/live-stream-egress-srs-deployment.yaml` | create |
| `tests/rtmp_egress_e2e.rs` | create |

## Code Shape

```rust
// usecase: StartLiveStreamEgress
pub struct StartLiveStreamEgress;
impl StartLiveStreamEgress {
    pub async fn execute(&self, ctx: &Ctx, host: &Principal, instance_id: &InstanceId, destination: EgressDestination) -> Result<EgressHandle> {
        // Cedar gate
        ctx.cedar.require(Action::StartLiveStreamEgress, host, instance_id).await?;
        // Tenant allow-list check
        if !ctx.tenant_policy.egress_destination_allowed(host.tenant_id(), &destination).await? {
            return Err(Error::EgressDestinationNotAllowed(destination.host()));
        }
        // Retrieve stream key from OpenBao
        let stream_key = ctx.secrets.fetch(&format!("meet/{}/egress/{}", host.tenant_id(), instance_id)).await?;
        // Spawn SRS outbound RTMP publish
        let handle = ctx.srs.publish_outbound(instance_id, &destination, &stream_key).await?;
        ctx.audit.seal(LiveStreamEgressStarted {
            instance_id: instance_id.clone(),
            destination: destination.host().to_string(),
            started_by: host.id(),
        }).await?;
        Ok(handle)
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-meet-live-stream-egress-adapter-srs
cargo nextest run --test rtmp_egress_e2e
cargo run -p oya-dev-cli -- gate validate egress-allow-list --microservice meet
```

## Test Plan

- RTMP outbound smoke: SRS publishes to a mock-RTMP-server; ffprobe verifies stream.
- WHIP fallback: when platform doesn't accept RTMP, WHIP handshake completes.
- Egress allow-list deny: destination not on tenant list → refused; metric incremented.
- Cedar deny: non-host attempts to start egress → refused.
- DNS allow-list: NetworkPolicy refuses outbound DNS to non-listed host.

## Halt Conditions

- NetworkPolicy missing egress allow-list — refuse.
- Stream key stored anywhere outside OpenBao — refuse.

## Next IP

[`IP-012-e2e-encryption-mls.md`](IP-012-e2e-encryption-mls.md)

## References

- ADR-MEET-0004 (live-streaming egress policy).
- RTMP spec (Adobe legacy; current de-facto for YouTube/Twitch).
- WHIP/WHEP IETF drafts.
- SRS RTMP `github.com/ossrs/srs`.
