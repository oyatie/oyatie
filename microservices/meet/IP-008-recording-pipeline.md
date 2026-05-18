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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: recording pipeline (LiveKit egress → ffmpeg gVisor → S3)

## Intent

Author the recording BC: LiveKit egress streams composite (audio + video + screen-share) to ffmpeg recording worker (running under gVisor `runtimeClassName: gvisor` per ADR-MEET-0002). ffmpeg muxes to MKV/MP4 with per-segment integrity (content_hash); finalised blob written to S3 with SSE-KMS tenant-DEK envelope encryption + Object Lock (WORM) per SEC Rule 17a-4(f); recording manifest in Postgres; `RecordingFinalized` event emitted to audit-chain.

Retention floors per pack (HIPAA 6y; SEC 17a-4 3-7y; MiFID II 5-7y; KR PIPA Art. 21 1-5y) applied via retention worker.

### Cross-µservice handoff to the `recordings` carrier

Per ADR-0140 (retired per ADR-0145) (cross-cutting-carriers adapter exemption) AND the
Workflow+Ontology adapter rule (feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145)),
the meet → recordings handoff MUST flow through the workflow-engine event-bus
for the orchestration leg, not via direct gRPC from a `meet` worker to the
`recordings` namespace:

1. When a meeting ends, the meet `recording` worker emits
   `meet.meeting.v1.ended` (carrying meeting_id + tenant_id + retention_policy_id
   + legal_hold_state) into workflow-engine.
2. workflow-engine evaluates the recording-trigger workflow and, when the
   tenant has recording enabled, emits `recordings.ingest.v1.requested` to
   the `recordings` µservice.
3. `recordings` consumes the workflow-engine event and pulls the muxed media
   from meet's S3 bucket using a signed URL whose grant is bounded by
   `recordings.ingest.v1.requested.expires_at` (typically T+24h). No direct
   gRPC channel from meet to recordings is opened.
4. `recordings` emits `recordings.ingest.v1.completed` (or `.failed`) back via
   workflow-engine; meet's `recording` BC observes completion via the
   workflow-engine subscription and updates its recording manifest accordingly.

The NetworkPolicy at `iac/helm/meet/templates/networkpolicy.yaml` egresses
ONLY to the `workflow-engine` namespace for this handoff; the `recordings`
namespace egress was removed 2026-05-18 per the integration review INT-002
finding.

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
- ADR-0140 (cross-cutting-carriers adapter exemption — clarifies that the
  meet → recordings handoff orchestration leg flows through workflow-engine
  even though `recordings` is a cross-cutting carrier; the carrier-exemption
  permits direct binary-payload pull via signed URL but the trigger remains
  workflow-engine-mediated).
- gVisor `gvisor.dev`.
- ffmpeg `ffmpeg.org`.
- SEC Rule 17a-4(f); HIPAA §164.312(c)(1); MiFID II RTS 6.
- S3 Object Lock docs (OCI / AWS-compatible).
- `iac/helm/meet/templates/networkpolicy.yaml` (egress to workflow-engine
  namespace only; the `recordings` namespace egress was removed 2026-05-18
  per the integration review INT-002 finding).
