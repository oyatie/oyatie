---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-004-recording-bc
status: pending
owner: axis-recordings
acceptance_lanes: [port-location, lean-a1]
---

# IP-004: Recording BC — kernel + domain + usecase + REST (read-side)

## Intent

Land the recording-asset shape + manifest + chapter-index + speaker-index
kernel + REST surface for list / get / metadata-update.

## Concrete crates

`oya-recordings-recording-{kernel,domain,usecase,api,adapter-postgres,adapter-s3,rest,worker,sdk,app}`.

## Acceptance Gates

```bash
cargo nextest run -p oya-recordings-recording-kernel
cargo run -p oya-dev-cli -- gate validate port-location --microservice recordings
```

## Next IP

[`IP-005-media-segment-bc.md`](IP-005-media-segment-bc.md)
