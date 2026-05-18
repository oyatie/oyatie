---
doc_class: ImplementationPlan
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-007-audio-track-library-and-attribution-bc
status: pending
owner: axis-shorts + ops-legal
depends_on: [IP-003]
---

# IP-007: audio-track-library + audio-attribution BC end-to-end

## Intent

- `audio-track-library` BC: licensed audio catalog + UGC sounds; per-pack licensing metadata; tenant-uploaded original sounds.
- `audio-attribution` BC: per-video sound attribution; rights chain; sound-of-the-week derivation source.

## ChangeSet boundary

7 + 6 = 13 crates: `oya-shorts-audio-track-library-{...}` + `oya-shorts-audio-attribution-{...}`.

## Concrete File Targets

Key entities: `AudioTrack`, `LicensedTrack`, `UgcSound`, `LicensingTier`, `AudioAttribution`, `SoundUsage`, `RightsChain`.

Ports: `AudioTrackRepository`, `AttributionStore`, `LicensorRegistry`.

Per-pack licensing metadata:
- pack-kr: KMRA / KMCA licensors.
- pack-eu: GEMA (DE) / SACEM (FR) / PRS for Music (UK) / SUISA (CH) — depending on tenant location.
- pack-us: ASCAP / BMI / SESAC.
- pack-jp: JASRAC.
- pack-br: ECAD.
- Generic free-tier: royalty-free + Creative Commons + UGC.

## Acceptance Gates

```bash
cargo build -p oya-shorts-audio-track-library-rest
cargo nextest run -p oya-shorts-audio-track-library-{kernel,domain,usecase,adapter-postgres,adapter-s3}
cargo nextest run -p oya-shorts-audio-attribution-{kernel,domain,usecase,adapter-postgres}
```

E2E: search sound → attach to video → rights-chain emitted → audit-chain seal.

## Halt Conditions

- Licensor metadata schema drift — engage ops-legal.

## Next IP

[`IP-008-feed-timeline-and-watch-time-bc.md`](IP-008-feed-timeline-and-watch-time-bc.md)

## References

- PRD FR-05, FR-06.
- `compliance.md` §pack-overrides for licensor mapping.
