---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-002-cargo-workspace-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-recordings
acceptance_lanes: [layer-correctness, port-location, lean-a1, per-microservice-layout]
---

# IP-002: Cargo workspace bootstrap (22 BC crate families)

## Intent

Scaffold the 22 bounded-context crate families under `microservices/recordings/src/crates/`
per ADR-0131 + ADR-0105 13-layer enum + ADR-0106 usecase rename.

## ChangeSet boundary

One cohesive ChangeSet: per-BC kernel crate skeletons (empty + naming-
conformant + cargo metadata). Subsequent IPs flesh out each BC.

## Crate families created

22 BCs × layers (kernel mandatory; others per BC):

- recording: kernel, domain, usecase, api, adapter-postgres, adapter-s3, rest, worker, sdk, app
- media-segment: kernel, domain, usecase, adapter-s3, adapter-cdn-cloudfront-stub-or-self, adapter-ffmpeg, worker, app
- transcript: kernel, domain, usecase, api, adapter-postgres, adapter-whisper, adapter-pyannote, rest, worker, sdk, app
- translation: kernel, domain, usecase, api, adapter, worker, sdk, app
- redaction: kernel, domain, usecase, api, adapter-postgres, adapter-ffmpeg, rest, worker, sdk, app
- chapter-marker: kernel, domain, usecase, api, adapter-postgres, rest, worker, sdk, app
- summary: kernel, domain, usecase, api, adapter-postgres, adapter-whisper, rest, worker, sdk, app
- thumbnail-pack: kernel, domain, usecase, adapter-s3, adapter-ffmpeg, worker, app
- search: kernel, domain, usecase, api, adapter-meilisearch, rest, sdk, app
- retention-policy: kernel, domain, usecase, api, adapter-postgres, rest, worker, sdk, app
- legal-hold: kernel, domain, usecase, api, adapter-postgres, rest, worker, sdk, app
- export: kernel, domain, usecase, api, adapter-ffmpeg, adapter-pandoc, worker, sdk, app
- share-link: kernel, domain, usecase, api, adapter-postgres, adapter-redis, rest, worker, sdk, app
- playback: kernel, domain, usecase, api, adapter-cdn-cloudfront-stub-or-self, adapter-redis, rest, sdk, app
- ediscovery: kernel, domain, usecase, api, adapter-postgres, worker, sdk, app
- watermarking: kernel, domain, usecase, adapter-ffmpeg, worker, app
- drm-stub: kernel, domain, usecase, adapter, app
- audio-loudness: kernel, domain, usecase, adapter-ffmpeg, worker, app
- video-encode-ladder: kernel, domain, usecase, adapter-ffmpeg, worker, app
- accessibility-captions: kernel, domain, usecase, api, adapter-postgres, rest, sdk, app
- recording-ingest: kernel, domain, usecase, api, adapter, adapter-s3, rest, worker, sdk, app

## Acceptance Gates

```bash
cargo build --workspace
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice recordings
cargo run -p oya-dev-cli -- gate validate port-location --microservice recordings
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice recordings
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice recordings
```

## Next IP

[`IP-003-recording-ingest-bc.md`](IP-003-recording-ingest-bc.md)

## References

- ADR-0056, ADR-0105, ADR-0106, ADR-0131.
- `PRD.md` Bounded Contexts table.
