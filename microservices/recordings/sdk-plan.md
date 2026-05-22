---
doc_class: SdkPlan
template_id: TPL-SDK
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-recordings + ops-developer-experience
related_adrs: [ADR-0131, ADR-RECORDINGS-0004, ADR-RECORDINGS-0007]
doc_status: published
---

# SDK Plan: recordings µservice

## Surfaces

| SDK | Audience | Substrate |
|---|---|---|
| `oya-recordings-*-sdk` (Rust) | internal oyatie µservices | trait re-exports from kernel + REST client |
| TypeScript SDK | Workflow Studio shell + web/desktop clients | OpenAPI codegen + WebSocket protocol |
| Swift SDK | iOS mobile client | OpenAPI codegen + HLS player wrapper |
| Kotlin SDK | Android mobile client | OpenAPI codegen + ExoPlayer wrapper |
| Python SDK | external API consumers + data-science teams | OpenAPI codegen |
| Go SDK | partner ingestion services | OpenAPI codegen |

## Capabilities Exposed

| Capability | SDK surface | Auth |
|---|---|---|
| List recordings | `recordings.list(filter)` | Cedar-scoped per-tenant |
| Get recording manifest | `recordings.get(id)` | Cedar |
| Start playback session | `recordings.playback.start(id, opts)` | Cedar + signed-URL |
| Read transcript | `recordings.transcript.get(id, lang?)` | Cedar |
| Search across recordings + transcripts | `recordings.search(query, filters)` | Cedar-scoped |
| Add redaction overlay | `recordings.redaction.add(id, span, reason)` | compliance-officer Cedar |
| Engage legal hold | `recordings.legalHold.engage(scope, courtOrderRef, paired)` | four-eyes |
| Release legal hold | `recordings.legalHold.release(holdId, reason, paired)` | four-eyes |
| Create share-link | `recordings.shareLink.create(id, opts)` | Cedar |
| Trigger export | `recordings.export.start(id, formats[])` | Cedar |
| Trigger eDiscovery export | `recordings.ediscovery.export(holdId, paired)` | compliance + four-eyes |
| Manual ingest (upload URL + finalize) | `recordings.ingest.presign()` + `recordings.ingest.finalize(uploadId)` | per-tenant rate-limited |
| Subscribe to events | WebSocket `recordings.events` | Cedar-filtered |

## Playback SDK (mobile / web)

- HLS player wrapper with chapter-skip / caption-toggle / speaker-filter /
  2x-speed.
- Adaptive bitrate (ABR) per RFC 8216 §4.3.4.10.
- Per-viewer watermark applied on playback (visible + steganographic
  invisible).
- Caption rendering per W3C WebVTT (`.vtt`) + TTML / EBU-TT-D (`.ttml`).
- Offline-download mode for tenant-admin-authorized share-links.

## Auth + Cedar Surface in SDKs

- All SDKs surface Cedar evaluation as the canonical authorization layer.
- SDKs do not embed Cedar policy locally; every request is server-side
  evaluated.

## SDK Versioning

- Per `feedback_no_silent_regression`: SDK major-version bump on breaking
  changes; 6-month sunset for old majors.
- SemVer 2.0.0.

## Generation Pipeline

```bash
cargo run -p oya-dev-cli -- sdk generate --microservice recordings --target {ts,swift,kotlin,python,go}
```

Generation drives off the OpenAPI + AsyncAPI + proto contracts under
`contracts/`. CI lane `sdk-conformance` runs baseline roundtrip tests per
target language.

## References

- ADR-0131.
- ADR-RECORDINGS-0004 (playback / HLS).
- ADR-RECORDINGS-0007 (ingest contract — manual-upload presign-URL pattern).
- `contracts/openapi/recordings.yaml`, `contracts/asyncapi/recordings-events.yaml`, `contracts/proto/recordings.proto`.
