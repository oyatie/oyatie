---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-004-asset-bcs-image-video-audio-adapters
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, gvisor-isolation, layer-correctness]
depends_on: [IP-002]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: image + video-embed + audio-embed BCs — adapters (S3 + ImageMagick + ffmpeg + ClamAV + OPSWAT)

## Intent

Author the asset BCs (image, video-embed, audio-embed) including backend-qualified adapters: S3 storage, ImageMagick 7.1 image processing, ffmpeg 7.x transcode, ClamAV + OPSWAT dual-scan upload pipeline (gVisor-sandboxed).

## ChangeSet boundary

Many crates per BC across kernel + domain + usecase + api + adapter + backend-qualified adapters.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-slides-image-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-imagemagick,adapter-clamav}/...` | create |
| `src/crates/oya-slides-video-embed-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-ffmpeg,adapter-clamav}/...` | create |
| `src/crates/oya-slides-audio-embed-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-ffmpeg}/...` | create |

## Code Shape

`image-adapter-clamav/src/lib.rs`:

```rust
pub struct ClamAvScanner { ... }

impl AssetScanner for ClamAvScanner {
    fn scan(&self, asset_bytes: &[u8]) -> Result<ScanVerdict, ScanError> {
        // Invoke clamd via Unix socket inside gVisor sandbox; return verdict.
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-slides-image-adapter-clamav --test scan_clean
cargo nextest run -p oya-slides-image-adapter-clamav --test scan_eicar  # known-malware EICAR test string
cargo nextest run -p oya-slides-video-embed-adapter-ffmpeg --test transcode_deterministic
oya gate validate gvisor-isolation --microservice slides
```

## Test Plan

| Test | Verifies |
|---|---|
| ClamAV EICAR detection | malware verdict emitted |
| OPSWAT secondary scan | dual-scanner verdict required if ClamAV unavailable |
| gVisor sandbox isolation | crash in transcode worker doesn't escape |
| ImageMagick determinism | identical input → identical output |

## Halt Conditions

- gVisor sandbox isolation test fails — STOP. Security-critical.

## Next IP

IP-005.
