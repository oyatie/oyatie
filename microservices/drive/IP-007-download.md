---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-007-download
status: pending
execution_unit: ChangeSet
owner: axis-drive
acceptance_lanes: [cargo-build, cargo-nextest]
---

# IP-007: download BC — range + signed URL + CDN steering

## Intent

Stand up `oya-drive-download-*` BC. Implement HTTP Range (RFC 9110), signed-URL minting, CDN steering, byte-range stream optimisation.

## Crates

`oya-drive-download-{kernel,domain,usecase,api,adapter,adapter-s3,rest,app}` (8 crates).

## Acceptance Gates

```bash
cargo nextest run --test e2e_range_download
cargo nextest run --test e2e_signed_url
cargo nextest run -p oya-drive-download-usecase -- first_byte_warm
```

## References

- PRD-drive §FR-02; AC-02.
- RFC 9110 (HTTP semantics; Range).
