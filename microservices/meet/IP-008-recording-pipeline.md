---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-008-recording-pipeline
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet + ops-security
acceptance_lanes: [cargo-nextest, recording-pipeline-smoke, oya-governance-gvisor-policy, oya-governance-s3-object-lock]
---

# IP-008: recording pipeline (LiveKit egress → ffmpeg gVisor → S3)

## Intent

Author the recording BC: LiveKit egress streams composite (audio + video + screen-share) to ffmpeg recording worker (running under gVisor `runtimeClassName: gvisor` per ADR-MEET-0002). ffmpeg muxes to MKV/MP4 with per-segment integrity (content_hash); finalised blob written to S3 with SSE-KMS tenant-DEK envelope encryption + Object Lock (WORM) per SEC Rule 17a-4(f); recording manifest in Postgres; `RecordingFinalized` event emitted to audit-chain.

Retention floors per pack (HIPAA 6y; SEC 17a-4 3-7y; MiFID II 5-7y; KR PIPA Art. 21 1-5y) applied via retention worker.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-meet-recording-{kernel,domain,usecase}/src/...` | create |
| `src/crates/oya-meet-recording-adapter-s3/src/client.rs` | create — multipart upload + SSE-KMS + Object Lock |
| `src/crates/oya-meet-recording-adapter-ffmpeg/src/mux.rs` | create — subprocess controller under gVisor |
| `src/crates/oya-meet-recording-adapter-postgres/src/manifest.rs` | create |
| `src/crates/oya-meet-recording-rest/src/handlers.rs` | create — start/stop/list/get/hold endpoints |
| `src/crates/oya-meet-recording-worker/src/egress_consumer.rs` | create — LiveKit egress event consumer |
| `iac/helm/meet/templates/recording-worker-deployment.yaml` | edit — `runtimeClassName: gvisor` |
| `tests/recording_pipeline_e2e.rs` | create |

## Code Shape

```rust
// adapter-ffmpeg/src/mux.rs
pub struct FfmpegMux { /* subprocess handle */ }

impl FfmpegMux {
    pub fn start(&self, input_url: &str, output_path: &Path) -> Result<()> {
        // ffmpeg subprocess under gVisor; capabilities dropped; read-only FS except scratch
        let mut cmd = Command::new("/usr/bin/ffmpeg");
        cmd.args([
            "-i", input_url,
            "-c:v", "libx264", "-preset", "fast", "-crf", "23",
            "-c:a", "aac", "-b:a", "128k",
            "-movflags", "+faststart",
            output_path.to_str().unwrap(),
        ]);
        // ...
        Ok(())
    }
}
```

```yaml
# recording-worker-deployment.yaml snippet
spec:
  template:
    spec:
      runtimeClassName: gvisor  # ADR-MEET-0002 sandbox requirement
      securityContext:
        runAsNonRoot: true
        seccompProfile: {type: RuntimeDefault}
      containers:
        - name: recording-worker
          securityContext:
            readOnlyRootFilesystem: true
            allowPrivilegeEscalation: false
            capabilities: {drop: ["ALL"]}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-meet-recording-adapter-s3
cargo nextest run -p oya-meet-recording-adapter-ffmpeg
cargo nextest run --test recording_pipeline_e2e
cargo run -p oya-dev-cli -- gate validate gvisor-policy --microservice meet
cargo run -p oya-dev-cli -- gate validate s3-object-lock --microservice meet
```

## Test Plan

- Recording start p95 ≤ 800ms; finalize on meeting-end within 30s for a 60min meeting.
- Object Lock verification: attempt to overwrite recording blob → S3 refuses.
- Tenant-DEK envelope: verify SSE-KMS key matches per-tenant key path.
- gVisor active: pod runtime is gVisor; sandbox escape attempts (synthetic) contained.
- Legal hold: hold opened → recording past retention floor preserved; hold closed → eligible for purge.

## Halt Conditions

- ffmpeg pod not running under gVisor — refuse merge.
- S3 bucket without Object Lock — refuse.
- Per-tenant DEK envelope not enforced — refuse.

## Next IP

[`IP-009-transcription-pipeline.md`](IP-009-transcription-pipeline.md)

## References

- ADR-MEET-0002 (recording + transcription pipeline).
- gVisor `gvisor.dev`.
- ffmpeg `ffmpeg.org`.
- SEC Rule 17a-4(f); HIPAA §164.312(c)(1); MiFID II RTS 6.
- S3 Object Lock docs (OCI / AWS-compatible).
