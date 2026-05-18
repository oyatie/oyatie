---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-011-import-export-pptx-pdf-mp4-pipeline
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, slides-pptx-roundtrip-subset, mp4-determinism, pdf-a-conformance, gvisor-isolation]
depends_on: [IP-004]
---

# IP-011: import-export BC — PPTX + ODP + PDF + Keynote + MP4 + PNG pipeline (gVisor sandboxed)

## Intent

Author the import-export BC per ADR-SLIDES-0003: Pandoc PPTX bridge (import) + bespoke OOXML serializer (export) + WeasyPrint/Chromium-headless PDF + ffmpeg MP4 + PNG-per-slide. All gVisor sandboxed.

## ChangeSet boundary

11 crates:
- `oya-slides-import-export-{kernel,domain,usecase,api,adapter,adapter-pandoc,adapter-weasyprint,adapter-chromium-headless,adapter-ffmpeg,worker,sdk}`

## Concrete File Targets

`src/crates/oya-slides-import-export-...`

## Code Shape

`import-export-adapter-ffmpeg/src/lib.rs`:

```rust
pub struct FfmpegMp4Exporter { ... }

impl Mp4Exporter for FfmpegMp4Exporter {
    fn export(&self, frames: &[PerSlidePng], audio: Option<&AudioStream>) -> Result<Mp4Output, ExportError> {
        // Deterministic-mode flags pinned per ADR-SLIDES-0003:
        //   -shortest -fflags +genpts -copyts -map_metadata -1
        // Output sha256 logged for determinism verification.
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.args(["-f", "image2pipe", "-r", "30",
                  "-shortest", "-fflags", "+genpts", "-copyts",
                  "-map_metadata", "-1",
                  "-c:v", "libx264",
                  "-pix_fmt", "yuv420p",
                  "-preset", "medium",
                  ...]);
        // Run inside gVisor sandbox.
        let output = self.gvisor.run(cmd)?;
        Ok(Mp4Output::from(output))
    }
}
```

`tests/integration/mp4_deterministic.rs`:

```rust
#[test]
fn test_mp4_determinism() {
    let frames = load_golden_frames("tests/golden/mp4/50-slide-frames/");
    let exporter = FfmpegMp4Exporter::new(gvisor());
    let out1 = exporter.export(&frames, None).unwrap();
    let out2 = exporter.export(&frames, None).unwrap();
    assert_eq!(sha256(out1.bytes()), sha256(out2.bytes()));
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-slides-import-export-domain --test pptx_roundtrip_subset
cargo nextest run -p oya-slides-import-export-adapter-ffmpeg --test mp4_determinism
cargo nextest run -p oya-slides-import-export-adapter-weasyprint --test pdf_a_conformance
oya gate validate slides-pptx-roundtrip-subset --microservice slides
oya gate validate mp4-determinism --microservice slides
oya gate validate pdf-a-conformance --microservice slides
oya gate validate gvisor-isolation --microservice slides
```

## Halt Conditions

- PPTX round-trip subset < 95% — STOP. AC-02 invariant.
- MP4 non-deterministic — STOP. ADR-SLIDES-0003 invariant.
- PDF/A non-conformance — STOP.
- gVisor sandbox escape — STOP. Sev-1 security.

## Next IP

IP-012.
