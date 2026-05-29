---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-010-presenter-audience-view-broadcast-mode-livekit
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + axis-realtime
acceptance_lanes: [cargo-check, cargo-nextest, broadcast-livekit-types-not-leaked, broadcast-speaker-notes-isolation]
depends_on: [IP-006]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: presenter-view + audience-view + broadcast-mode (LiveKit reuse via messenger)

## Intent

Author the present-mode BCs + broadcast-mode reusing messenger's LiveKit infrastructure per ADR-SLIDES-0005.

## ChangeSet boundary

~20 crates:
- `oya-slides-presenter-view-...`
- `oya-slides-audience-view-...`
- `oya-slides-broadcast-mode-{kernel,domain,usecase,api,adapter,adapter-livekit,worker,sdk}`
- `oya-slides-speaker-notes-{kernel,domain,usecase,api,adapter}`

## Concrete File Targets

`src/crates/oya-slides-presenter-view-...`, `oya-slides-audience-view-...`, `oya-slides-broadcast-mode-...`, `oya-slides-speaker-notes-...`

## Code Shape

`broadcast-mode-adapter-livekit/src/lib.rs`:

```rust
// LiveKit types are CONFINED to this crate per ADR-SLIDES-0005 + ADR-0105 Amd.3.
// Slides' own BroadcastSession / SignalRoute / ViewerLease entities wrap.

use messenger_sdk::LivekitClient;  // ONLY consumer of messenger SDK LiveKit binding.

pub struct LivekitBroadcastBridge {
    client: LivekitClient,
}

impl BroadcastSignalRouter for LivekitBroadcastBridge {
    async fn create_room(&self, session: &BroadcastSession) -> Result<RoomHandle, BroadcastError> {
        let room = self.client.create_room(/* pack-pinned, presenter-bound */).await?;
        Ok(RoomHandle::from_messenger(room))
    }
}
```

`broadcast-mode-domain/src/speaker_notes_isolation.rs`:

```rust
/// T-I-07 invariant: speaker-notes never crosses broadcast frame.
pub fn project_for_broadcast(deck: &Deck) -> BroadcastFrame {
    let mut frame = BroadcastFrame::from(deck);
    frame.strip_speaker_notes();  // Mandatory; tested.
    frame
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-slides-broadcast-mode-domain --test speaker_notes_excluded
cargo nextest run -p oya-slides-broadcast-mode-domain --test pack_residency_refused
cargo nextest run -p oya-slides-broadcast-mode-adapter-livekit --test livekit_types_confined
oya gate validate broadcast-livekit-types-not-leaked --microservice slides
oya gate validate broadcast-speaker-notes-isolation --microservice slides
```

## Halt Conditions

- Speaker-notes leak test fails — STOP. T-I-07 invariant.
- LiveKit type leaks past adapter — STOP. ADR-SLIDES-0005 invariant.

## Next IP

IP-011.
