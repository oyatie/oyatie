---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-002-cargo-workspace-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-shorts
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-bnf-v4-1, oya-governance-layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Cargo workspace bootstrap (~140 crates per ADR-0131 + ADR-0105 Amendment 3)

## Intent

Scaffold the ~140 crates for the shorts µservice per ADR-0131 per-microservice
flat layout + ADR-0105 13-value layer enum + ADR-0106 usecase rename +
ADR-0056 BNF v4.1 + ADR-0105 Amendment 3 backend-qualified -adapter-<backend>.

Crate families (per PRD §"Bounded Contexts"; ~22 BCs × ~6-8 layers):
- `oya-shorts-video-upload-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,rest,sdk,app}`
- `oya-shorts-video-transcode-{kernel,domain,usecase,api,adapter,adapter-ffmpeg,adapter-s3,worker,sdk}`
- `oya-shorts-video-storage-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-cloudfront,sdk}`
- `oya-shorts-thumbnail-generation-{kernel,domain,usecase,api,adapter,adapter-ffmpeg,adapter-s3,worker,sdk}`
- `oya-shorts-audio-track-library-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,sdk}`
- `oya-shorts-audio-attribution-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}`
- `oya-shorts-video-composition-{kernel,domain,usecase,api,adapter,adapter-ffmpeg,worker,sdk,app}`
- `oya-shorts-feed-timeline-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk,app}`
- `oya-shorts-watch-time-tracking-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}`
- `oya-shorts-like-share-comment-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}`
- `oya-shorts-repost-stitch-duet-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-ffmpeg,worker,sdk}`
- `oya-shorts-hashtag-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}`
- `oya-shorts-trending-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}`
- `oya-shorts-notifications-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk,app}`
- `oya-shorts-content-moderation-{kernel,domain,usecase,api,adapter,adapter-clamav,adapter-opswat,worker,sdk}`
- `oya-shorts-copyright-claim-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}`
- `oya-shorts-age-gate-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}`
- `oya-shorts-parental-controls-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}`
- `oya-shorts-accessibility-captions-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,worker,sdk}`
- `oya-shorts-creator-analytics-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}`
- `oya-shorts-monetization-stub-{kernel,domain,usecase,api,sdk}` (off by default)
- `oya-shorts-live-streaming-stub-{kernel,sdk}` (off in M03)
- `oya-shorts-drm-{kernel,domain,usecase,api,adapter,adapter-widevine,adapter-fairplay,adapter-playready,sdk}`

## ChangeSet boundary

Pure Cargo workspace registration + crate skeletons (Cargo.toml + lib.rs / main.rs) per crate; no business logic. Each kernel registers `data_class` annotations per Bominal ADR-0028. Subsequent IPs implement each BC.

## Concrete File Targets

| Path | Action |
|---|---|
| `Cargo.toml` (workspace) | extend `members` with ~140 entries |
| `crates/oya-shorts-<bc>-<layer>/Cargo.toml` | ~140 crates created |
| `crates/oya-shorts-<bc>-<layer>/src/lib.rs` | ~140 lib.rs files |
| `crates/oya-shorts-<bc>-app/src/main.rs` | per-BC app binaries (subset) |
| `crates/oya-shorts-<bc>-worker/src/main.rs` | per-BC worker binaries (subset) |
| `crates/oya-shorts-<bc>-rest/src/main.rs` | per-BC REST binaries (subset) |

## Crate Naming Justification

Each crate carries the standard preamble in its lib.rs / Cargo.toml header:

```
NAME: oya-shorts-<bc>-<layer>
JUSTIFICATION:
- microservice = shorts: per ADR-0131 per-microservice flat layout.
- bc-tokens = <bc>: PRD §"Bounded Contexts". ADR-0056 v4.1 BC-optionality rule honoured.
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-<backend> per ADR-0105 Amendment 3 (postgres, redis, s3,
  cloudfront, meilisearch, ffmpeg, clamav, opswat, widevine, fairplay, playready).
```

## Acceptance Gates

```bash
cargo build --workspace
cargo nextest run --workspace
cargo run -p oya-dev-cli -- gate validate bnf-v4-1 --microservice shorts
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice shorts
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice shorts
```

## Test Plan

- Per-crate: `cargo test -p <crate>` smoke (empty test set acceptable for skeleton).
- Workspace-wide `cargo nextest run`.
- Coverage threshold: not applicable at skeleton stage (subsequent IPs raise per-BC coverage).

## Halt Conditions

- BNF v4.1 lint fail — fix naming.
- Layer correctness lint fail — verify each crate's layer placement.
- Workspace bloat (build time > 5 min on standard runner) — split workspace per ADR-0131.

## Next IP

[`IP-003-video-upload-bc.md`](IP-003-video-upload-bc.md)

## References

- ADR-0056 BNF v4.1.
- ADR-0105 13-value layer enum + Amendment 3.
- ADR-0106 usecase rename.
- ADR-0131 per-µservice flat layout.
- `microservices/shorts/PRD.md` §Bounded Contexts.
