---
doc_class: MigrationGuide
template_id: TPL-MIGRATION-GUIDE
microservice: recordings
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-RECORDINGS accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0003, ADR-RECORDINGS-0004, ADR-RECORDINGS-0005, ADR-RECORDINGS-0006, ADR-RECORDINGS-0007]
related_specs: [/specs/microservices/recordings.json, /specs/microservices/recordings/recordings.json]
owner_team: axis-recordings
date: 2026-05-17
doc_status: published
---

# Migration: `oya-connect-recordings-*` → `oya-recordings-*`

This document applies the Strangler Pattern from the agent-skills
`deprecation-and-migration` skill to the **recordings** µservice. It is the
consumer-facing companion to ADR-0134 (cross-µservice migration policy) and
ADR-0135 (target topology).

## Status

**Deprecated as of 2026-05-17 — replacement available; new µservice scaffolded
in `microservices/recordings/`.**

| Field | Value |
|---|---|
| Replacement | `oya-recordings-*` crate family under `microservices/recordings/src/crates/` |
| Removal date | **Advisory** — concrete target is HG-RECORDINGS accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #2) |
| Reason | ADR-0132 no-suite forward-policy + ADR-0139 per-µservice SLO authority + ADR-0131 per-µservice flat layout + the 22-BC recordings surface (recording / media-segment / transcript / translation / redaction / chapter-marker / summary / thumbnail-pack / search / retention-policy / legal-hold / export / share-link / playback / ediscovery / watermarking / drm-stub / audio-loudness / video-encode-ladder / accessibility-captions / recording-ingest) is only addressable at µservice granularity |
| Migration owner (Churn Rule) | axis-recordings |
| Migration window | Phase 2 adapter + Phase 3 canary = ~5 months; Phase 5 removal sweep in month 6 (see ADR-0134) |

## Replacement

The 22 bounded-contexts of the `recordings` µservice live under
`microservices/recordings/src/crates/` per ADR-0131. The legacy
`oya-connect-recordings-domain` crate is **a single bundled crate** that
splits per BC; consumers must pick the specific replacement BC per
import-site.

### Crate import-path map

Legacy `oya-connect-recordings-domain` (single crate) →
`oya-recordings-{recording,media-segment,transcript,translation,redaction,chapter-marker,summary,thumbnail-pack,search,retention-policy,legal-hold,export,share-link,playback,ediscovery,watermarking,drm-stub,audio-loudness,video-encode-ladder,accessibility-captions,recording-ingest}-{kernel,domain,usecase,api,adapter-*,rest,worker,sdk,app}` (per BC, per layer).

| Legacy symbol | New crate · symbol |
|---|---|
| `oya_connect_recordings_domain::RecordingArchiveEntry` | `oya_recordings_recording_kernel::Recording` |
| `oya_connect_recordings_domain::RecordingArchiveEntryCreate` | `oya_recordings_recording_kernel::RecordingCreate` |
| `oya_connect_recordings_domain::ArchiveRetentionPolicy` | `oya_recordings_retention_policy_kernel::RetentionPolicy` |
| `oya_connect_recordings_domain::ArchiveRetentionPolicyCreate` | `oya_recordings_retention_policy_kernel::RetentionPolicyCreate` |
| `oya_connect_recordings_domain::RecordingVariant` | `oya_recordings_media_segment_kernel::MediaSegment` (HLS) + `oya_recordings_export_kernel::ExportVariant` (export-bundle) — splits per concern |
| `oya_connect_recordings_domain::RecordingVariantFormat` | `oya_recordings_media_segment_kernel::SegmentFormat` (HLS/DASH/CMAF) + `oya_recordings_export_kernel::ExportFormat` (mp4/mp3/wav/vtt/srt/pdf/docx) |
| `oya_connect_recordings_domain::RecordingArchiveReader` (port) | `oya_recordings_recording_kernel::RecordingRepository` |
| `oya_connect_recordings_domain::RecordingArchiveError` | `oya_recordings_recording_kernel::RecordingError` + per-BC variants |
| `oya_connect_recordings_domain::RecordingsSurfaceStaging` | retired — surface staging shape is now the `RecordingIngestRequest` in `oya_recordings_recording_ingest_kernel` per ADR-RECORDINGS-0007 |

### Net-new boundaries (no legacy counterpart)

The new µservice introduces capabilities that did NOT exist in
`oya-connect-recordings-domain`. They are therefore not part of the migration
surface — they are clean replacement-boundary features:

- **Per-µservice transcript BC** (`oya-recordings-transcript-*`) — legacy
  carried only an optional `transcript_ref` URI; the new µservice owns the
  transcript shape, diarization, redaction, translation.
- **Speaker diarization** (`oya-recordings-transcript-adapter-pyannote`) —
  pyannote 3.x; net-new per ADR-RECORDINGS-0001.
- **Redaction overlay** (`oya-recordings-redaction-*`) — overlay-model that
  does not mutate source media; net-new per ADR-RECORDINGS-0003.
- **Legal-hold load-bearing BC** (`oya-recordings-legal-hold-*`) — load-
  bearing 100 % correctness invariant; legacy had only an optional
  `legal_hold_id` field on the retention policy.
- **eDiscovery export bundle with chain-of-custody Merkle seal**
  (`oya-recordings-ediscovery-*`) — net-new per ADR-RECORDINGS-0002.
- **Per-viewer dynamic + steganographic watermark**
  (`oya-recordings-watermarking-*`) — net-new per ADR-RECORDINGS-0004.
- **HLS multi-bitrate ladder + CMAF segmentation**
  (`oya-recordings-video-encode-ladder-*` + `oya-recordings-media-segment-*`)
  — net-new per ADR-RECORDINGS-0004.
- **Multi-source ingest contract** (`oya-recordings-recording-ingest-*`) —
  durable contract for meet + huddles + manual + live-stream → recordings;
  net-new per ADR-RECORDINGS-0007. Legacy only handled meet-source
  recordings via `RecordingArchiveEntryCreate` direct-write.
- **Tiered storage substrate** (hot s3 + cold s3-glacier-class) — net-new
  per ADR-RECORDINGS-0005.
- **Cross-µservice translate handoff** (`oya-recordings-translation-*`) —
  net-new; bridge to `translate` µservice.

### Concrete import migration recipes

```rust
// BEFORE
use oya_connect_recordings_domain::{
    RecordingArchiveEntry, RecordingArchiveEntryCreate,
    ArchiveRetentionPolicy, ArchiveRetentionPolicyCreate,
    RecordingVariant, RecordingVariantFormat,
    RecordingArchiveError, RecordingArchiveReader,
};

// AFTER
use oya_recordings_recording_kernel::{
    Recording, RecordingCreate, RecordingError, RecordingRepository,
};
use oya_recordings_retention_policy_kernel::{
    RetentionPolicy, RetentionPolicyCreate,
};
use oya_recordings_media_segment_kernel::{
    MediaSegment, SegmentFormat,           // HLS/DASH/CMAF
};
use oya_recordings_export_kernel::{
    ExportVariant, ExportFormat,            // mp4/mp3/wav/vtt/srt/pdf/docx
};
use oya_recordings_recording_ingest_kernel::{
    RecordingIngestRequest, IngestSourceKind,
};
```

```toml
# BEFORE
[dependencies]
oya-connect-recordings-domain = { workspace = true }

# AFTER
[dependencies]
oya-recordings-recording-kernel          = { workspace = true }
oya-recordings-retention-policy-kernel   = { workspace = true }
oya-recordings-media-segment-kernel      = { workspace = true }
oya-recordings-export-kernel             = { workspace = true }
oya-recordings-recording-ingest-kernel   = { workspace = true }
```

## Reason

The legacy `oya-connect-recordings-domain` crate was authored before the
following ADRs crystallised:

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle
   membership at the architecture layer; bundle membership is a brand-layer
   concept and must not appear in crate names.
2. **ADR-0139 — per-µservice SLO authority.** Recordings needs independent
   SLO targets per surface (recording-list, playback-start, transcript-search,
   transcript-render, redaction-render, export-mp4, export-transcript-pdf,
   legal-hold-engagement + zero-tolerance load-bearing invariants for
   retention-policy-correctness and legal-hold-chain-of-custody-correctness).
3. **ADR-0131 — per-µservice flat layout.** Recordings's IaC, runbooks,
   threat-model, DPIA, compliance, capacity-model, cost-budget, incident-
   response, failure-modes, multi-region all need to live under one folder
   (`microservices/recordings/`).
4. **ADR-0133 — 11-pack-overlay program.** pack-kr (KR PIPA + KR 전자문서법),
   pack-eu (GDPR Art. 17), pack-us (general), pack-us-healthcare (HIPAA
   recordings PHI), pack-us-financial (SEC 17a-4(f) + FINRA 4511 + MiFID II
   recorded-communications retention), pack-jp (APPI), pack-sg (PDPA), pack-au
   (Privacy Act + TIA Act), pack-in (DPDPA), pack-br (LGPD), pack-ae (UAE
   PDPL), pack-ksa (KSA PDPL) — each lives as
   `microservices/recordings/iac/kustomize/overlays/pack-<region>/` and per-
   pack regulatory overlay sections in threat-model.md / dpia.md /
   compliance.md.
5. **ADR-RECORDINGS-0001 → ADR-RECORDINGS-0007** — recordings-specific
   decisions (transcription pipeline, retention + legal-hold, redaction
   overlay, playback + CDN, tiered storage, AI feature bounds, multi-source
   ingest contract) need to live at per-µservice ADR granularity, not at
   the Connect suite level.
6. **`recordings` is now a centralised store** receiving from meet +
   messenger + future live-broadcast + manual upload. The legacy
   `oya-connect-recordings-domain` only modelled meet-source recordings via
   `RecordingRef`/`RecordingArchiveEntryCreate`; the new µservice owns the
   multi-source ingest contract per ADR-RECORDINGS-0007.

## Migration Guide (step-by-step)

For each consumer crate that imports `oya-connect-recordings-domain`:

### Step 1 — Add the new dependencies

In your consumer crate's `Cargo.toml`, add the new mapped dependencies (per
the `[dependencies]` snippet above; pick only the BCs you actually use).
Keep the legacy dependency for now (Phase 2 adapter soak).

### Step 2 — Update imports per the import-path map above

```bash
rg -l "oya_connect_recordings_domain" --type rust path/to/your/crate
```

Manual disambiguation needed for the `RecordingVariant` / `RecordingVariantFormat`
split (HLS-segment vs. export-bundle); the migration adapter shim provides
guard rails on both shapes during the soak.

### Step 3 — Verify behavioural parity

```bash
cargo nextest run --features connect-recordings-strangler-canary
```

Run with the feature flag enabled to route through the new µservice; run
without to route through the legacy adapter. Compare:

- error variant ordering (Hyrum's Law — see surfaces below).
- p99 latency (must be ≤ legacy + 5 % per ADR-0134 Phase 3 canary gate).
- log-line format (preserved verbatim during the canary; may be tightened
  in a successor-IP `feedback_no_silent_regression`-conforming ADR).

### Step 4 — Remove the legacy dependency

Only after your consumer crate's tests pass against the new imports AND the
recordings µservice's Phase 3 canary reaches 100 % traffic (per ADR-0134),
remove the legacy dependency.

```toml
# Remove:
oya-connect-recordings-domain = { workspace = true }
```

### Step 5 — Verify zero residual

```bash
cargo tree -e normal -p your-crate | grep oya-connect-recordings   # expect empty
rg "use oya_connect_recordings_" --type rust path/to/your/crate    # expect zero
```

## Configuration delta

| Configuration key | Legacy | New |
|---|---|---|
| Feature flag namespace | `connect.recordings.*` | `recordings.*` |
| OpenSLO file | bundled in `Connect.openslo.yaml` (umbrella) | `microservices/recordings/slos/*.openslo.yaml` (per-µservice, 10 files) |
| Helm chart values key | `.Values.connect.recordings.*` | `.Values.recordings.*` |
| K8s namespace | `connect` | `recordings` |
| Cedar policy fragment path | `policy/connect/recordings/*.cedar` | `microservices/recordings/policy/cedar/*.cedar` |
| pack-kr overlay path | `policy/connect/recordings/pack-kr/*` | `microservices/recordings/iac/kustomize/overlays/pack-kr/*` |
| Workflow event prefix | `connect.recordings.*` | `recordings.*` (e.g., `recordings.recording.published.v1`, `recordings.transcript.ready.v1`, `recordings.legalhold.engaged.v1`) |
| Ontology type prefix | `Connect.Recording.*` | `Recordings.*` (e.g., `Recordings.Recording`, `Recordings.Transcript`, `Recordings.LegalHold`) |
| Telemetry metric prefix | `oya_connect_recordings_*` | `oya_recordings_*` |
| Tracing span attribute namespace | `connect.recordings.*` | `recordings.*` |
| CDN backend choice | (legacy implicit CloudFront) | per ADR-RECORDINGS-0004 — `recordings.iac.helm.cloudfront.*` (primary) + `recordings.iac.helm.self-host.*` (pack-cn / pack-ksa) |
| Storage tier policy | (legacy hot S3 only) | per ADR-RECORDINGS-0005 — hot S3 + cold s3-glacier-class with per-pack age-down policy |
| Whisper engine pin | (legacy — none; transcript was opaque) | `whisper-large` via foundry-runtime per ADR-RECORDINGS-0001 |
| Diarization engine pin | (legacy — none) | `pyannote-3.x` via foundry-runtime per ADR-RECORDINGS-0001 |
| ffmpeg engine pin | (legacy implicit `ffmpeg`) | `ffmpeg-7.x` in gVisor sandbox per ADR-RECORDINGS-0004 |

## Hyrum's-Law surfaces — explicit callouts

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law Makes Removal
Hard", these are the legacy recordings surfaces with observable behaviour
that may be depended on. Each is preserved verbatim during the canary;
consumers must re-test after Phase 5 removal in case they had a long-tail
dependency:

1. **HLS manifest byte stability.** Legacy emitted the `RecordingVariant`
   `storage_key` as `tenant/meet/session-/rec-/<variant>.<ext>`; the new
   µservice's `oya-recordings-media-segment-adapter-s3` preserves the same
   key shape for the meet-source variant during the canary. After the
   canary, the canonical shape is
   `tenant/<tenant_id>/source-<source_kind>/<recording_id>/<bitrate>/<segment_index>.m4s`.
   The HLS manifest itself preserves segment-byte-range deterministic
   ordering — if a consumer pattern-matches on byte offsets of the manifest,
   it will continue to match within ±64 bytes (segment-byte-range encoded
   under CMAF chunking).
2. **Transcript JSON field ordering.** Legacy `transcript_ref` pointed at a
   blob with serialised fields in lexicographic order. The new
   `oya-recordings-transcript-kernel::Transcript` JSON renderer emits in the
   same lexicographic order during the canary. Consumers that pattern-match
   on the field-order see no change. After the canary, the canonical
   ordering is `[segment_index, speaker_id, start_ms, end_ms, text, confidence, redacted_spans]` —
   if a consumer depended on lexicographic order, it must switch to
   schema-aware decoders (serde) before Phase 5.
3. **Redaction-overlay coordinate inclusivity.** Legacy did not have a
   redaction-overlay shape. The new `oya-recordings-redaction-kernel::RedactionSpan`
   uses `[start_ms, end_ms)` half-open. **Explicit callout** because there
   is no legacy contract here — consumers implementing redaction overlays
   for the first time MUST adopt half-open semantics.
4. **Signed-URL TTL behaviour.** Legacy emitted signed URLs with a fixed
   1h TTL. The new `oya-recordings-share-link-kernel::ShareLink` defaults
   to 24h TTL with tenant-configurable up to 7d. Consumers depending on the
   1h cliff for cache eviction MUST switch to the share-link's `expires_at`
   field. Documented in `runbooks/playback-cdn-cache-cascade.md`.
5. **Retention purge cliff.** Legacy `can_purge_at(now)` returned true when
   `legal_hold_id.is_none() && now >= purge_after_epoch_seconds`. The new
   `oya-recordings-retention-policy-kernel::RetentionPolicy::can_purge_at`
   preserves the exact same semantics during the canary, with the
   strengthening that **load-bearing 100 % correctness** is now CI-enforced.
   After the canary, the canonical purge cliff includes a 7-day soft grace
   to allow human intervention; the soft grace is observable as an event
   `RetentionPurgePending` 7 days before the hard cliff.
6. **KMS-shred ordering.** Legacy did not emit `KmsShredExecuted`. The new
   µservice emits this audit-chain event. Consumers that did not have to
   observe shred ordering before may now observe it; not a Hyrum's-Law risk
   per se, but flagged for completeness.
7. **`RecordingsSurfaceStaging` shape retirement.** The legacy
   `RecordingsSurfaceStaging` struct (recording_id + tenant_id +
   kms_shred_ref) is retired; the new ingest-side shape is
   `RecordingIngestRequest` (richer; includes source_kind, audit-chain
   parent, expected content-hash). The legacy shape is preserved in the
   migration adapter shim during Phase 2/3 for byte-compat.

## Runbook continuity table

| Legacy operational surface | New runbook (under `microservices/recordings/runbooks/`) | Status |
|---|---|---|
| (legacy — meet-only retention purge) | `retention-policy-rollback.md` | NEW + strengthened per ADR-RECORDINGS-0002 |
| (no legacy counterpart) | `legal-hold-court-order-receipt.md` | NEW per ADR-RECORDINGS-0002 + court-order workflow |
| (no legacy counterpart) | `transcript-pipeline-degraded-whisper.md` | NEW per ADR-RECORDINGS-0001 |
| (no legacy counterpart) | `redaction-overlay-corruption.md` | NEW per ADR-RECORDINGS-0003 |
| (no legacy counterpart) | `playback-cdn-cache-cascade.md` | NEW per ADR-RECORDINGS-0004 |
| (no legacy counterpart) | `transcode-pipeline-failure.md` | NEW per ADR-RECORDINGS-0004 + ffmpeg gVisor |
| (no legacy counterpart) | `ediscovery-export.md` | NEW per ADR-RECORDINGS-0002 + chain-of-custody Merkle seal |
| (no legacy counterpart) | `watermark-key-rotation.md` | NEW per ADR-RECORDINGS-0004 |

## Phases (per ADR-0134)

| Phase | Description | Status (recordings) | Exit condition |
|---|---|---|---|
| 1. Parallel ship | New µservice + legacy coexist | **active** | HG-RECORDINGS passes at p99 SLOs in dev cluster sustained 7d |
| 2. Adapter soak | `oya-connect-recordings-migration-adapter` shims legacy symbols → new impl | pending | All consumers compile against adapter; 3-month soak elapses |
| 3. Feature-flagged canary | 10 % → 50 % → 100 % traffic shift over 6 weeks | pending | New µservice carries 100 % traffic for 7 consecutive days |
| 4. Zero-active-usage verification | Dependency-graph + telemetry + grep all clean | pending | Verification commands all exit 0 |
| 5. Code removal sweep | Delete legacy crate + Cargo.toml entries + spec pointers | pending | `cargo build --workspace` exits 0; no `oya_connect_recordings_*` symbol resolves |
| 6. Umbrella retirement | Conditional on all 8 sub-µservices reaching their own Phase 5 | pending | All 8 HG-<MS> gates green at p99 SLO sustained 30d |

## Verification checklist (per skill SKILL.md §"Verification")

- [ ] **Replacement is production-proven and covers all critical use cases.**
  ```bash
  cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice recordings
  ```
- [ ] **Migration guide exists with concrete steps and examples.**
  ```bash
  test -f microservices/recordings/migration-from-connect.md
  ```
- [ ] **All active consumers have been migrated** (per Phase 4):
  ```bash
  cargo tree -e normal -p oya-connect-recordings-domain --invert    | grep -v 'oya-connect-recordings-migration-adapter' | wc -l   # expect 0
  rg "use oya_connect_recordings_" --type rust    | rg -v "migration-adapter|legacy_in_process|tests/"    | wc -l   # expect 0
  ```
- [ ] **Old code, tests, documentation, configuration removed** (per Phase 5):
  ```bash
  find crates -maxdepth 1 -type d -name "oya-connect-recordings-*" | wc -l    # expect 0
  ```
- [ ] **No references to the deprecated system remain** (excluding historical
  ADR / RETIRED.md / git-log):
  ```bash
  rg "oya_connect_recordings" --type rust    | rg -v "docs/decisions/|RETIRED.md|tests/baseline/"    | wc -l   # expect 0
  ```
- [ ] **Deprecation notices removed (they served their purpose)** (Phase 5):
  ```bash
  test ! -f microservices/recordings/deprecation-notice.md
  test ! -f microservices/recordings/migration-from-connect.md
  ```

## Breaking changes (flagged per `feedback_no_silent_regression`)

Phases 1–4: **not breaking** for the core symbol surface — the adapter
preserves the legacy `RecordingArchiveEntry` / `ArchiveRetentionPolicy` /
`RecordingVariant` shapes verbatim, including error-variant ordering and
timing within the +5 % canary tolerance.

**There IS one behavioural strengthening** that may visibly differ from
legacy and is NOT preserved by the adapter (per
`feedback_no_silent_regression`):

1. **Retention-policy correctness + legal-hold chain-of-custody correctness
   are now load-bearing 100 % invariants** (per ADR-RECORDINGS-0002 + this
   PRD). Consumers that depended on eventually-consistent legal-hold
   engagement (e.g., "legal-hold takes effect within 5 minutes") will now
   see p99 ≤ 1s engagement; any breach is a Sev-1 audit event. This is a
   deliberate strengthening; consumers should remove any retry-windows
   anticipating eventual consistency.

Phase 5 (code removal) **IS a breaking change** for any consumer that did
not migrate during the 5-month adapter + canary window. Per
`feedback_no_silent_regression`:

- Sunset schedule (advisory): 6 months from this document's
  `deprecation_date` (2026-05-17), so a target advisory removal date of
  **2026-11-17** (subject to the HG-RECORDINGS retirement trigger gating).
- Owning axis (axis-recordings) ships migration ChangeSets for every
  internal consumer per the Churn Rule before Phase 5.
- External consumers (reading `/specs/microservices/recordings.json`)
  receive a 6-month sunset window from this notice.

## References

- ADR-0135: Connect super-app expansion into flat µservices.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-suite forward-policy.
- ADR-0133: Industry best-practice conformance program.
- ADR-0134: Connect dissolution Strangler migration (operational policy).
- ADR-RECORDINGS-0001..0007.
- RFC 8216 — HLS.
- ISO/IEC 23009-1 — DASH.
- ISO/IEC 23000-19 — CMAF.
- W3C WebVTT, W3C TTML, EBU-TT-D.
- ISO/IEC 14496-12 — MP4.
- RFC 6716 — Opus.
- EBU R128 — audio loudness.
- SMPTE-TT timed text.
- `microservices/recordings/PRD.md` — full target-state product definition.
- `microservices/recordings/PHASE-01-RECORDINGS-FOUNDATION.md` — phase plan.
- `microservices/recordings/runbooks/*.md` — 8 runbooks.
- `microservices/recordings/deprecation-notice.md` — formal deprecation notice.
- `feedback_no_silent_regression.md` — no-silent-regression principle.
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern + Adapter Pattern + Churn Rule + Verification.
